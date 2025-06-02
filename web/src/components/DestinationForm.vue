<template>
  <div flex flex-col items-center justify-center gap-4>
    <h1 text-2xl>Set Destination</h1>
    <el-form :model="form" label-width="auto" label-position="left" style="max-width: 360px">
      <el-form-item label="Cluster Seed">
        <el-input v-model="form.seed" />
      </el-form-item>
      <el-form-item label="Number of Stars">
        <el-input v-model="form.stars" />
      </el-form-item>
      <el-form-item label="Resource Multiplier">
        <el-input v-model="form.resource" />
      </el-form-item>
      <el-form-item label=" ">
        <span>Metadata Output Multiplier {{ OutputMultiplier }}%</span>
      </el-form-item>
      <el-form-item label="Cluster Address">
        <span>
          {{ address }}
        </span>
      </el-form-item>
    </el-form>
  </div>
</template>
<script setup lang="ts">
import { reactive } from "vue";

// do not use same name with ref
const form = reactive({
  seed: "",
  stars: 64,
  resource: 1,
});

const OutputMultiplier = computed(() => {
  return form.resource * 100;
});
const address = computed(() => {
  return "0x1234567890abcdef";
});
</script>
<style lang="scss">
.el-form {
  --el-form-label-font-size: 18px;
}
</style>
