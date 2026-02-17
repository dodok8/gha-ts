<script setup lang="ts">
defineProps<{
  background?: string
}>()
</script>

<template>
  <div class="slidev-layout cover text-center" :style="background ? { background } : {}">
    <div class="my-auto w-full">
      <slot />
    </div>
  </div>
</template>
