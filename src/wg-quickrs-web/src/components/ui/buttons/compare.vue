<template>
  <button :disabled="disabled"
          :class="active ? 'bg-gray-600 text-white' : ''"
          class="disabled:opacity-40 bg-gray-100 hover:enabled:bg-gray-600 text-gray-600 hover:enabled:text-white p-1 rounded"
          :title="title"
          @click="$emit('click');">
    <!-- "merge-cells" @ https://flowbite.com/icons/ -->
    <svg :class="imageClasses" class="fill-none" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
         width="24" height="24" viewBox="0 0 24 24">
      <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M10 18v2H4V4h6v2m4 12v2h6V4h-6v2m-6.49543 8.4954L10 12m0 0L7.50457 9.50457M10 12H4.05191m12.50199 2.5539L14 12m0 0 2.5539-2.55392M14 12h5.8319"/>
    </svg>
  </button>
</template>
<script>
export default {
  name: "compare-button",
  props: {
    disabled: false,
    active: false,
    title: "",
    imageClasses: "",
  },
  emits: ['click'],
}
</script>

<style scoped>

</style>