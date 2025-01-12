<template>

  <div :class="[computed_colors.div]" class="p-1 shadow-md border rounded">
    <div class="text-gray-800 mb-0.5">
      Configure Script Snippets:
    </div>
    <div v-for="field in ['pre_up', 'post_up', 'pre_down', 'post_down']">
      <div
          :class="[scripts_local[field].enabled !== scripts[field].enabled || scripts_local[field].value !== scripts[field].value ? 'highlight-undo-box' : '']"
          class="form-check truncate flex items-center relative mb-0.5">
        <label class="flex-none">
          <input
              :checked="scripts_local[field].enabled"
              class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 mt-1 align-top bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
              type="checkbox"
              @change="scripts_local[field].enabled = !scripts_local[field].enabled;">
          <span class="text-gray-800 cursor-pointer text-xs mr-1">
                       <strong class="text-sm">{{ SCRIPTS_KEY_LOOKUP[field] }}:</strong>
                      </span>
        </label>
        <input
            v-model="scripts_local[field].value"
            :class="[`enabled:${computed_colors[field]}`]" :disabled="!scripts_local[field].enabled"
            :placeholder="`${SCRIPTS_KEY_LOOKUP[field]} Script (e.g. echo 'Hey, this is ${SCRIPTS_KEY_LOOKUP[field]} Script';)`"
            class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow disabled:bg-gray-100"
            type="text"/>
        <div
            v-if="!(scripts_local[field].enabled === scripts[field].enabled && scripts_local[field].value === scripts[field].value)"
            class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
          <button
              :disabled="scripts_local[field].enabled === scripts[field].enabled && scripts_local[field].value === scripts[field].value"
              class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
              title="Undo Changes"
              @click="scripts_local[field].enabled = scripts[field].enabled; scripts_local[field].value = scripts[field].value;">
            <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
          </button>
        </div>
      </div>
    </div>
  </div>

</template>

<script>
import WireGuardHelper from "../../js/wg-helper.js";


export default {
  name: "scripts-island",
  props: {
    scripts: {
      type: Object,
      default: {
        pre_up: {enabled: false, value: ""},
        post_up: {enabled: false, value: ""},
        pre_down: {enabled: false, value: ""},
        post_down: {enabled: false, value: ""},
      }
    },
    defaultScripts: {
      type: Object,
      default: {
        pre_up: {enabled: false, value: ""},
        post_up: {enabled: false, value: ""},
        pre_down: {enabled: false, value: ""},
        post_down: {enabled: false, value: ""},
      },
    },
    value: {
      type: Object,
      default: {
        context: 'edit',
        changedFields: {},
        error: null,
      },
    },
  },
  data() {
    return {
      scripts_local: {},
      SCRIPTS_KEY_LOOKUP: {
        pre_up: 'PreUp',
        post_up: 'PostUp',
        pre_down: 'PreDown',
        post_down: 'PostDown',
      },
    };
  },
  created() {
    this.scripts_local = this.scripts;
  },
  computed: {
    computed_colors() {
      const changedFields = {
        scripts: {
          pre_up: {}, post_up: {}, pre_down: {}, post_down: {},
        },
      };
      let error = null;
      const colors = {};
      for (const field of ['pre_up', 'post_up', 'pre_down', 'post_down']) {
        colors[field] = 'bg-white';
        if (this.value.context === 'create' ||
            this.scripts_local[field].enabled !== this.scripts[field].enabled ||
            this.scripts_local[field].value !== this.scripts[field].value) {
          colors[field] = WireGuardHelper.checkField(field, this.scripts_local[field]) ? 'bg-green-200' : 'bg-red-200';
        }

        if (this.scripts_local[field].enabled !== this.scripts[field].enabled) changedFields.scripts[field].enabled = this.scripts_local[field].enabled;
        if (this.scripts_local[field].value !== this.scripts[field].value) changedFields.scripts[field].value = this.scripts_local[field].value;
        if (Object.keys(changedFields.scripts[field]).length === 0) delete changedFields.scripts[field];

        error = this.scripts_local[field].enabled && colors[field] === 'bg-red-200' ? field : error;
      }
      if (Object.keys(changedFields.scripts).length === 0) delete changedFields.scripts;

      colors.div = 'bg-gray-100';
      if (this.scripts_local.pre_up.enabled
          || this.scripts_local.post_up.enabled
          || this.scripts_local.pre_down.enabled
          || this.scripts_local.post_down.enabled) {
        colors.div = 'bg-green-50';
        if ((this.scripts_local.pre_up.enabled && colors.pre_up === 'bg-red-200')
            || (this.scripts_local.post_up.enabled && colors.post_up === 'bg-red-200')
            || (this.scripts_local.pre_down.enabled && colors.pre_down === 'bg-red-200')
            || (this.scripts_local.post_down.enabled && colors.post_down === 'bg-red-200')) {
          colors.div = 'bg-red-50';
        }
      }
      this.value.changedFields = changedFields;
      this.value.error = error;

      return colors;
    },
  },
}
</script>

<style scoped>

</style>