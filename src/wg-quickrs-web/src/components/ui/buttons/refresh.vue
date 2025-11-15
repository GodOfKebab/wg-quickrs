<template>
  <button class="align-middle rounded bg-gray-100 text-gray-600 hover:bg-gray-600 hover:text-gray-100 transition-all mx-2"
          :title="title"
          @click="$emit('click');">
    <!-- "refresh" @ https://flowbite.com/icons/ -->
    <svg class="w-6 h-6 fill-none" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
         width="24" height="24" viewBox="0 0 24 24">
      <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M17.651 7.65a7.131 7.131 0 0 0-12.68 3.15M18.001 4v4h-4m-7.652 8.35a7.13 7.13 0 0 0 12.68-3.15M6 20v-4h4"/>
    </svg>
  </button>
</template>
<script>
export default {
  name: "refresh-button",
  props: {
    title: "",
  },
  emits: ['click'],
}
</script>

<style scoped>

</style>