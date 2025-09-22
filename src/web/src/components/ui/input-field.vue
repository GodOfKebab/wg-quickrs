<template>
  <div class="my-0.5 truncate flex items-center relative ml-2">
    <!-- Label -->
    <checkbox v-if="isEnabledValue" :checked="modelValue.enabled" :label="`${label}:`" class="mr-1" size="5.5"
              @click="$emit('update:modelValue',{enabled: !modelValue.enabled, value: modelValue.value})"></checkbox>
    <field v-else :field="`${label}:`" class="mr-1"></field>


    <!-- Input -->
    <input
        :disabled="disabled || (isEnabledValue ? !modelValue.enabled : false)"
        :class="[inputColor]"
        :list="`${label}-list`"
        :placeholder="placeholder"
        :value="isEnabledValue ? modelValue.value : modelValue"
        class="rounded pl-1.5 pt-[2px] pb-[1px] my-0.5 focus:outline-none focus:ring-0 border-1 border-gray-200 focus:border-gray-400 outline-none w-full text-lg text-gray-500 grow disabled:bg-gray-100"
        type="text"
        @input="$emit('update:modelValue', isEnabledValue ? {enabled: modelValue.enabled, value: $event.target.value} : $event.target.value)"/>

    <!-- Undo Button -->
    <undo-button v-if="!_fast_equal(modelValue, valuePrev) && !disabled"
                 :disabled="_fast_equal(modelValue, valuePrev)"
                 alignment-classes="right-[5px] top-[4.5px]"
                 @click="$emit('update:modelValue', valuePrev);">
    </undo-button>
  </div>

</template>

<script>
import FastEqual from "fast-deep-equal";
import UndoButton from "@/components/ui/undo-button.vue";
import Checkbox from "@/components/ui/checkbox.vue";
import Field from "@/components/ui/field.vue";

export default {
  name: "input-field",
  components: {Field, Checkbox, UndoButton},
  props: {
    modelValue: null,
    valuePrev: null,
    label: "",
    placeholder: "",
    inputColor: "",
    disabled: false,
    isEnabledValue: false
  },
  emits: ['update:modelValue'],
  methods: {
    _fast_equal(s1, s2) {
      return FastEqual(s1, s2);
    },
  },
}
</script>

<style scoped>

</style>