<template>
  <div class="inline-flex items-center shrink-0">
    <label class="flex items-center cursor-pointer relative">
      <!-- Checkbox -->
      <input
          :disabled="disabled"
          :checked="checked"
          :class="`h-${size} w-${size}`"
          type="checkbox"
          @change="$emit('click');"/>

      <!-- Checkmark Icon -->
      <span v-if="checked"
            class="absolute text-white opacity-100 top-13/24 left-0.5 transform -translate-y-1/2 pointer-events-none">
        <svg :class="`h-${size-1} w-${size-1}`" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
          <path clip-rule="evenodd"
                d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                fill="currentColor" fill-rule="evenodd"></path>
        </svg>
      </span>

      <!-- Label text -->
      <field :field="label" class="ml-2"></field>
    </label>
  </div>
</template>

<script>
import Field from "@/components/ui/field.vue";

export default {
  name: "checkbox",
  components: {Field},
  props: {
    checked: false,
    disabled: false,
    label: "",
    size: 6
  },
  emits: ['click'],
}
</script>

<style scoped>

</style>