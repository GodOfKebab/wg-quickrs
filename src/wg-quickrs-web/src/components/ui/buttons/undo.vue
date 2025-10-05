<template>
  <div :class="alignmentClasses" class="inline-block float-right absolute z-20">
    <button
        :disabled="disabled"
        class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
        title="Undo Changes"
        @click="$emit('click');">
      <!-- "undo" @ https://flowbite.com/icons/ -->
      <svg :class="imageClasses" class="fill-none" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
           width="24" height="24" viewBox="0 0 24 24">
        <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M3 9h13a5 5 0 0 1 0 10H7M3 9l4-4M3 9l4 4"/>
      </svg>
    </button>
  </div>
</template>

<script>
export default {
  name: "undo-button",
  props: {
    disabled: false,
    alignmentClasses: "",
    imageClasses: "",
  },
  emits: ['click'],
}
</script>

<style scoped>

</style>