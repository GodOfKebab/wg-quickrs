<template>
  <button :disabled="disabled"
          :class="active ? 'bg-gray-600 text-white' : ''"
          class="disabled:opacity-40 bg-gray-100 hover:enabled:bg-gray-600 text-gray-600 hover:enabled:text-white p-1 rounded"
          :title="title"
          @click="$emit('click');">
    <!-- "file-code" @ https://flowbite.com/icons/ -->
    <svg :class="imageClasses" class="fill-none" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
         width="24" height="24" fill="none" viewBox="0 0 24 24">
      <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M10 3v4a1 1 0 0 1-1 1H5m5 4-2 2 2 2m4-4 2 2-2 2m5-12v16a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V7.914a1 1 0 0 1 .293-.707l3.914-3.914A1 1 0 0 1 9.914 3H18a1 1 0 0 1 1 1Z"/>
    </svg>
  </button>
</template>
<script>
export default {
  name: "conf-button",
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