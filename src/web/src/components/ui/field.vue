<template>
  <span class="text-gray-800 flex items-center">
    <strong class="text-lg mt-[1px]">{{ field }}</strong>
  </span>
</template>

<script>
export default {
  name: "field",
  props: {
    field: "",
  },
  emits: ['click'],
}
</script>

<style scoped>

</style>