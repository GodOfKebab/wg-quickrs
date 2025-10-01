<template>
  <button :disabled="disabled"
          :class="active ? 'bg-gray-600 text-white' : ''"
          class="disabled:opacity-40 bg-gray-100 hover:enabled:bg-gray-600 text-gray-600 hover:enabled:text-white p-1 rounded"
          :title="title"
          @click="$emit('click');">
    <!-- "download" @ https://flowbite.com/icons/ -->
    <svg :class="imageClasses" class="fill-none" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
         width="24" height="24" viewBox="0 0 24 24">
      <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M12 13V4M7 14H5a1 1 0 0 0-1 1v4a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-4a1 1 0 0 0-1-1h-2m-1-5-4 5-4-5m9 8h.01"/>
    </svg>
  </button>
</template>
<script>
export default {
  name: "download-button",
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