/// <reference types="vite/client" />

// 由 vite.config.ts 中 define 定义的全局常量
declare const __APP_VERSION__: string;

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
