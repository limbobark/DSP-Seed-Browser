<template>
  <div>
  </div>
</template>
<script setup lang="ts">
defineOptions({
  name: 'IndexPage',
})

const router = useRouter()

onMounted(() => {
  router.push(`/viewer`)
})
</script>


