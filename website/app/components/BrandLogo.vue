<script setup lang="ts">
const asset = usePublicAsset()
</script>
<template>
  <span class="brand-logo">
    <img :src="asset('argui-icon.png')" width="32" height="32" alt="" aria-hidden="true" />
    <span class="brand-word">argui</span>
  </span>
</template>
