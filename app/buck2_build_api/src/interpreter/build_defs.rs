/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use buck2_interpreter::path::StarlarkPath;
use buck2_interpreter_for_build::interpreter::build_context::BuildContext;
use buck2_interpreter_for_build::interpreter::functions::host_info::register_host_info;
use buck2_interpreter_for_build::interpreter::functions::read_config::register_read_config;
use buck2_interpreter_for_build::interpreter::natives::register_module_natives;
use buck2_interpreter_for_build::super_package::package_value::register_read_package_value;
use either::Either;
use itertools::Itertools;
use starlark::collections::SmallMap;
use starlark::docs::DocString;
use starlark::docs::DocStringKind;
use starlark::environment::GlobalsBuilder;
use starlark::eval::Evaluator;
use starlark::values::Value;
use starlark_map::small_set::SmallSet;

use crate::interpreter::rule_defs::provider::callable::UserProviderCallable;
use crate::interpreter::rule_defs::transitive_set::TransitiveSetDefinition;
use crate::interpreter::rule_defs::transitive_set::TransitiveSetError;
use crate::interpreter::rule_defs::transitive_set::TransitiveSetOperations;
use crate::interpreter::rule_defs::transitive_set::TransitiveSetProjectionKind;
use crate::interpreter::rule_defs::transitive_set::TransitiveSetProjectionSpec;

#[derive(Debug, thiserror::Error)]
enum NativesError {
    #[error("non-unique field names: [{}]", .0.iter().map(|s| format!("`{}`", s)).join(", "))]
    NonUniqueFields(Vec<String>),
    #[error("`transitive_set()` can only be used in `bzl` files")]
    TransitiveSetOnlyInBzl,
}

#[starlark_module]
pub fn register_provider(builder: &mut GlobalsBuilder) {
    /// Create a `"provider"` type that can be returned from `rule` implementations.
    /// Used to pass information from a rule to the things that depend on it.
    /// Typically named with an `Info` suffix.
    ///
    /// ```python
    /// GroovyLibraryInfo(fields = [
    ///     "objects",  # a list of artifacts
    ///     "options",  # a string containing compiler options
    /// ])
    /// ```
    ///
    /// Given a dependency you can obtain the provider with `my_dep[GroovyLibraryInfo]`
    /// which returns either `None` or a value of type `GroovyLibraryInfo`.
    ///
    /// For providers that accumulate upwards a transitive set is often a good choice.
    fn provider(
        #[starlark(require=named, default = "")] doc: &str,
        #[starlark(require=named)] fields: Either<Vec<String>, SmallMap<&str, &str>>,
        eval: &mut Evaluator,
    ) -> anyhow::Result<UserProviderCallable> {
        let docstring = DocString::from_docstring(DocStringKind::Starlark, doc);
        let path = BuildContext::from_context(eval)?.starlark_path().path();

        let (field_names, field_docs) = match fields {
            Either::Left(f) => {
                let docs = vec![None; f.len()];
                let field_names: SmallSet<String> = f.iter().cloned().collect();
                if field_names.len() != f.len() {
                    return Err(NativesError::NonUniqueFields(f).into());
                }
                (field_names, docs)
            }
            Either::Right(fields_with_docs) => {
                let mut field_names = SmallSet::with_capacity(fields_with_docs.len());
                let mut field_docs = Vec::with_capacity(fields_with_docs.len());
                for (name, docs) in fields_with_docs {
                    let inserted = field_names.insert(name.to_owned());
                    assert!(inserted);
                    field_docs.push(DocString::from_docstring(DocStringKind::Starlark, docs));
                }
                (field_names, field_docs)
            }
        };
        Ok(UserProviderCallable::new(
            path.into_owned(),
            docstring,
            field_docs,
            field_names,
        ))
    }
}

#[starlark_module]
pub fn register_transitive_set(builder: &mut GlobalsBuilder) {
    fn transitive_set<'v>(
        args_projections: Option<SmallMap<String, Value<'v>>>,
        json_projections: Option<SmallMap<String, Value<'v>>>,
        reductions: Option<SmallMap<String, Value<'v>>>,
        eval: &mut Evaluator,
    ) -> anyhow::Result<TransitiveSetDefinition<'v>> {
        let build_context = BuildContext::from_context(eval)?;
        // TODO(cjhopman): Reductions could do similar signature checking.
        let projections: SmallMap<_, _> = args_projections
            .into_iter()
            .flat_map(|v| v.into_iter())
            .map(|(k, v)| {
                (
                    k,
                    TransitiveSetProjectionSpec {
                        kind: TransitiveSetProjectionKind::Args,
                        projection: v,
                    },
                )
            })
            .chain(
                json_projections
                    .into_iter()
                    .flat_map(|v| v.into_iter())
                    .map(|(k, v)| {
                        (
                            k,
                            TransitiveSetProjectionSpec {
                                kind: TransitiveSetProjectionKind::Json,
                                projection: v,
                            },
                        )
                    }),
            )
            .collect();

        // Both kinds of projections take functions with the same signature.
        for (name, spec) in projections.iter() {
            // We should probably be able to require that the projection returns a parameters_spec, but
            // we don't depend on this type-checking and we'd just error out later when calling it if it
            // were wrong.
            if let Some(v) = spec.projection.parameters_spec() {
                if v.len() != 1 {
                    return Err(TransitiveSetError::ProjectionSignatureError {
                        name: name.clone(),
                    }
                    .into());
                }
            };
        }

        Ok(TransitiveSetDefinition::new(
            match build_context.starlark_path() {
                StarlarkPath::LoadFile(import_path) => import_path.clone(),
                _ => return Err(NativesError::TransitiveSetOnlyInBzl.into()),
            },
            TransitiveSetOperations {
                projections,
                reductions: reductions.unwrap_or_default(),
            },
        ))
    }
}

/// Natives for `BUCK` and `bzl` files.
pub(crate) fn register_build_bzl_natives(builder: &mut GlobalsBuilder) {
    register_provider(builder);
    register_transitive_set(builder);
    register_module_natives(builder);
    register_host_info(builder);
    register_read_config(builder);
    register_read_package_value(builder);
}

#[cfg(test)]
mod tests {
    use buck2_common::package_listing::listing::testing::PackageListingExt;
    use buck2_common::package_listing::listing::PackageListing;
    use buck2_core::build_file_path::BuildFilePath;
    use buck2_core::bzl::ImportPath;
    use buck2_interpreter::file_loader::LoadedModules;
    use buck2_interpreter::path::OwnedStarlarkModulePath;
    use buck2_interpreter_for_build::interpreter::natives::register_module_natives;
    use buck2_interpreter_for_build::interpreter::testing::cells;
    use buck2_interpreter_for_build::interpreter::testing::run_simple_starlark_test;
    use buck2_interpreter_for_build::interpreter::testing::Tester;
    use buck2_node::attrs::inspect_options::AttrInspectOptions;
    use buck2_node::nodes::unconfigured::testing::targets_to_json;
    use indoc::indoc;
    use serde_json::json;

    use crate::interpreter::build_defs::register_provider;
    use crate::interpreter::rule_defs::register_rule_defs;

    #[test]
    fn prelude_is_included() -> anyhow::Result<()> {
        let mut tester = Tester::new()?;
        let prelude_path = ImportPath::testing_new("root//prelude:prelude.bzl");
        tester.set_prelude(prelude_path.clone());

        let prelude =
            tester.eval_import(&prelude_path, "some_var = 1", LoadedModules::default())?;
        let mut loaded_modules = LoadedModules::default();
        loaded_modules
            .map
            .insert(OwnedStarlarkModulePath::LoadFile(prelude_path), prelude);

        // The prelude should be included in build files, and in .bzl files that are not in the
        // prelude's package
        let build_file = BuildFilePath::testing_new("root//prelude:TARGETS.v2");
        assert!(
            tester
                .eval_build_file_with_loaded_modules(
                    &build_file,
                    "other_var = some_var",
                    loaded_modules.clone(),
                    PackageListing::testing_empty()
                )
                .is_ok(),
            "build files in the prelude package should have access to the prelude"
        );

        let import = ImportPath::testing_new("root//not_prelude:sibling.bzl");
        assert!(
            tester
                .eval_import(&import, "other_var = some_var", loaded_modules.clone())
                .is_ok(),
            ".bzl files not in the prelude package should have access to the prelude"
        );

        let import = ImportPath::testing_new("root//prelude:defs.bzl");
        assert!(
            tester
                .eval_import(&import, "other_var = some_var", loaded_modules)
                .is_err(),
            "bzl files in the prelude package should NOT have access to the prelude"
        );

        Ok(())
    }

    #[test]
    fn test_package_import() -> anyhow::Result<()> {
        let mut tester = Tester::with_cells(cells(Some(indoc!(
            r#"
            [buildfile]
                package_includes = src=>//include.bzl::func_alias=some_func
        "#
        )))?)?;
        tester.additional_globals(register_rule_defs);
        tester.additional_globals(register_module_natives);

        let import_path = ImportPath::testing_new("root//:include.bzl");
        tester.add_import(
            &import_path,
            indoc!(
                r#"
            def _impl(ctx):
                pass
            export_file = rule(impl=_impl, attrs = {})

            def some_func(name):
                export_file(name = name)
        "#
            ),
        )?;

        let build_path = BuildFilePath::testing_new("root//src/package:BUCK");
        let eval_result = tester.eval_build_file(
            &build_path,
            indoc!(
                r#"
                implicit_package_symbol("func_alias")(
                    implicit_package_symbol("missing", "DEFAULT")
                )
                "#
            ),
            PackageListing::testing_files(&["file1.java", "file2.java"]),
        )?;
        assert_eq!(build_path.package(), eval_result.package());
        assert_eq!(
            json!({
                    "DEFAULT": {
                        "__type__": "root//include.bzl:export_file",
                        "compatible_with": [],
                        "default_target_platform": null,
                        "exec_compatible_with": [],
                        "name": "DEFAULT",
                        "target_compatible_with": [],
                        "tests": [],
                        "visibility": [],
                    },
            }),
            targets_to_json(
                eval_result.targets(),
                build_path.package(),
                AttrInspectOptions::All
            )?
        );
        Ok(())
    }

    #[test]
    fn test_provider() -> anyhow::Result<()> {
        // TODO: test restricting field names
        let mut tester = Tester::new().unwrap();
        tester.additional_globals(register_provider);
        tester.run_starlark_test(indoc!(
            r#"
            SomeInfo = provider(fields=["x", "y"])
            SomeOtherInfo = provider(fields={"x": "docs for x", "y": "docs for y"})
            DocInfo = provider(doc="Some docs", fields=["x", "y"])

            def test():
                instance = SomeInfo(x = 2, y = True)
                assert_eq(2, instance.x)
                assert_eq(True, instance.y)
                assert_eq(SomeInfo(x = 2, y = True), instance)

                instance = SomeOtherInfo(x = 2, y = True)
                assert_eq(2, instance.x)
                assert_eq(True, instance.y)
                assert_eq(SomeOtherInfo(x = 2, y = True), instance)

                instance = DocInfo(x = 2, y = True)
                assert_eq(2, instance.x)
                assert_eq(True, instance.y)
                assert_eq(DocInfo(x = 2, y = True), instance)
            "#
        ))?;
        Ok(())
    }

    #[test]
    fn eval() -> anyhow::Result<()> {
        let mut tester = Tester::new()?;
        tester.additional_globals(register_module_natives);
        tester.additional_globals(register_rule_defs);
        let content = indoc!(
            r#"
            def _impl(ctx):
                pass
            export_file = rule(impl=_impl, attrs = {})

            def test():
                assert_eq("some/package", __internal__.package_name())
                assert_eq("@root", __internal__.repository_name())

                assert_eq(package_name(), __internal__.package_name())
                assert_eq(repository_name(), __internal__.repository_name())

                assert_eq(package_name(), get_base_path())

                export_file(name = "rule_name")
                assert_eq(True, rule_exists("rule_name"))
                assert_eq(False, rule_exists("not_rule_name"))

                print("some message")
                print("multiple", "strings")
            "#
        );
        tester.run_starlark_test(content)?;
        Ok(())
    }

    #[test]
    fn test_internal() -> anyhow::Result<()> {
        // Test that most things end up on __internal__
        let mut tester = Tester::new().unwrap();
        tester.additional_globals(register_rule_defs);
        run_simple_starlark_test(indoc!(
            r#"
            def test():
                assert_eq(__internal__.json.encode({}), "{}")
            "#
        ))
    }

    #[test]
    fn test_oncall() -> anyhow::Result<()> {
        let mut tester = Tester::new().unwrap();
        tester.additional_globals(register_module_natives);
        tester.additional_globals(register_rule_defs);
        tester.run_starlark_test(indoc!(
            r#"
            def _impl(ctx):
                pass
            export_file = rule(impl=_impl, attrs = {})

            def test():
                oncall("valid")
                export_file(name = "rule_name")
            "#
        ))?;
        tester.run_starlark_test_expecting_error(
            indoc!(
                r#"
            def test():
                oncall("valid")
                oncall("twice")
            "#
            ),
            "more than once",
        );
        tester.run_starlark_test_expecting_error(
            indoc!(
                r#"
            def _impl(ctx):
                pass
            export_file = rule(impl=_impl, attrs = {})

            def test():
                export_file(name = "rule_name")
                oncall("failure after")
            "#
            ),
            "after one or more targets",
        );
        Ok(())
    }
}
