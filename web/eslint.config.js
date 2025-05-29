import pluginVue from "eslint-plugin-vue";
import standard from "@vue/eslint-config-standard";

export default [
  ...pluginVue.configs["flat/essential"],
  ...standard,
]
