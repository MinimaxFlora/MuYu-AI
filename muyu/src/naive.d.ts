import type {
  NConfigProvider,
  NDialogProvider,
  NInput,
  NMessageProvider,
  NSlider,
  NSpin,
  NTooltip,
} from "naive-ui";

declare module "vue" {
  export interface GlobalComponents {
    NConfigProvider: typeof NConfigProvider;
    NDialogProvider: typeof NDialogProvider;
    NMessageProvider: typeof NMessageProvider;
    NInput: typeof NInput;
    NTooltip: typeof NTooltip;
    NSlider: typeof NSlider;
    NSpin: typeof NSpin;
  }
}

export {};
