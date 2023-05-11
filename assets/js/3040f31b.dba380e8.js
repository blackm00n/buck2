"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[39],{3905:(e,t,r)=>{r.r(t),r.d(t,{MDXContext:()=>l,MDXProvider:()=>d,mdx:()=>g,useMDXComponents:()=>u,withMDXComponents:()=>s});var n=r(67294);function o(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function a(){return a=Object.assign||function(e){for(var t=1;t<arguments.length;t++){var r=arguments[t];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(e[n]=r[n])}return e},a.apply(this,arguments)}function i(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,n)}return r}function p(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?i(Object(r),!0).forEach((function(t){o(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):i(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function c(e,t){if(null==e)return{};var r,n,o=function(e,t){if(null==e)return{};var r,n,o={},a=Object.keys(e);for(n=0;n<a.length;n++)r=a[n],t.indexOf(r)>=0||(o[r]=e[r]);return o}(e,t);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);for(n=0;n<a.length;n++)r=a[n],t.indexOf(r)>=0||Object.prototype.propertyIsEnumerable.call(e,r)&&(o[r]=e[r])}return o}var l=n.createContext({}),s=function(e){return function(t){var r=u(t.components);return n.createElement(e,a({},t,{components:r}))}},u=function(e){var t=n.useContext(l),r=t;return e&&(r="function"==typeof e?e(t):p(p({},t),e)),r},d=function(e){var t=u(e.components);return n.createElement(l.Provider,{value:t},e.children)},m="mdxType",f={inlineCode:"code",wrapper:function(e){var t=e.children;return n.createElement(n.Fragment,{},t)}},b=n.forwardRef((function(e,t){var r=e.components,o=e.mdxType,a=e.originalType,i=e.parentName,l=c(e,["components","mdxType","originalType","parentName"]),s=u(r),d=o,m=s["".concat(i,".").concat(d)]||s[d]||f[d]||a;return r?n.createElement(m,p(p({ref:t},l),{},{components:r})):n.createElement(m,p({ref:t},l))}));function g(e,t){var r=arguments,o=t&&t.mdxType;if("string"==typeof e||o){var a=r.length,i=new Array(a);i[0]=b;var p={};for(var c in t)hasOwnProperty.call(t,c)&&(p[c]=t[c]);p.originalType=e,p[m]="string"==typeof e?e:o,i[1]=p;for(var l=2;l<a;l++)i[l]=r[l];return n.createElement.apply(null,i)}return n.createElement.apply(null,r)}b.displayName="MDXCreateElement"},74004:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>c,contentTitle:()=>i,default:()=>u,frontMatter:()=>a,metadata:()=>p,toc:()=>l});var n=r(87462),o=(r(67294),r(3905));const a={id:"bootstrapping",title:"Bootstrapping Buck2"},i="Bootstrapping Buck2",p={unversionedId:"bootstrapping",id:"bootstrapping",title:"Bootstrapping Buck2",description:"To generate BUCK files for buck2's dependencies, we use reindeer.",source:"@site/../docs/bootstrapping.md",sourceDirName:".",slug:"/bootstrapping",permalink:"/docs/bootstrapping",draft:!1,tags:[],version:"current",frontMatter:{id:"bootstrapping",title:"Bootstrapping Buck2"},sidebar:"manualSidebar",previous:{title:"Benefits When Compared to Buck1",permalink:"/docs/benefits"},next:{title:"Concept Map",permalink:"/docs/concepts/concept_map"}},c={},l=[],s={toc:l};function u(e){let{components:t,...r}=e;return(0,o.mdx)("wrapper",(0,n.Z)({},s,r,{components:t,mdxType:"MDXLayout"}),(0,o.mdx)("h1",{id:"bootstrapping-buck2"},"Bootstrapping Buck2"),(0,o.mdx)("p",null,"To generate ",(0,o.mdx)("inlineCode",{parentName:"p"},"BUCK")," files for ",(0,o.mdx)("inlineCode",{parentName:"p"},"buck2"),"'s dependencies, we use ",(0,o.mdx)("a",{parentName:"p",href:"https://github.com/facebookincubator/reindeer"},"reindeer"),"."),(0,o.mdx)("p",null,"Note that the resulting binary will be compiled without optimisations or ",(0,o.mdx)("a",{parentName:"p",href:"https://github.com/jemalloc/jemalloc"},"jemalloc"),", so we recommend using the Cargo-produced binary in further development."),(0,o.mdx)("p",null,"First, install ",(0,o.mdx)("inlineCode",{parentName:"p"},"reindeer")," with ",(0,o.mdx)("inlineCode",{parentName:"p"},"Cargo"),":"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre",className:"language-sh"},"cargo install --git https://github.com/facebookincubator/reindeer reindeer\n")),(0,o.mdx)("p",null,"Next, run the following to pull in dependencies and buckify:"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre",className:"language-sh"},"cd buck2/\nreindeer --third-party-dir shim/third-party/rust vendor\nreindeer --third-party-dir shim/third-party/rust buckify\n")),(0,o.mdx)("p",null,"Build ",(0,o.mdx)("inlineCode",{parentName:"p"},"buck2")," with ",(0,o.mdx)("inlineCode",{parentName:"p"},"buck2"),":"),(0,o.mdx)("pre",null,(0,o.mdx)("code",{parentName:"pre",className:"language-sh"},"buck2 build //:buck2\n")))}u.isMDXComponent=!0}}]);