<template>

  <div :class="[color_div]" class="p-1 shadow-md border rounded">
    <div class="text-gray-800 mb-0.5">
      Configure Script Snippets:
    </div>
    <div v-for="field in ['pre_up', 'post_up', 'pre_down', 'post_down']">
      <div
          class="form-check truncate flex items-center relative mb-0.5">
        <label class="form-check-label flex items-center">
          <input
              :checked="peer_local.scripts[field].enabled"
              class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
              type="checkbox"
              @change="peer_local.scripts[field].enabled = !peer_local.scripts[field].enabled;">
          <span class="text-gray-800 cursor-pointer text-xs mr-1">
            <strong class="text-sm">{{ SCRIPTS_KEY_LOOKUP[field] }}:</strong>
          </span>
        </label>
        <input
            v-model="peer_local.scripts[field].value"
            :class="[`enabled:${FIELD_COLOR_LOOKUP[is_changed_script_field[field]]}`]"
            :disabled="!peer_local.scripts[field].enabled"
            :placeholder="`${SCRIPTS_KEY_LOOKUP[field]} Script (e.g. echo 'Hey, this is ${SCRIPTS_KEY_LOOKUP[field]} Script';)`"
            class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow disabled:bg-gray-100"
            type="text"/>
        <div
            v-if="is_changed_script_field[field]"
            class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
          <button
              :disabled="!is_changed_script_field[field]"
              class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
              title="Undo Changes"
              @click="peer_local.scripts[field] = JSON.parse(JSON.stringify(peer.scripts[field]))">
            <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
          </button>
        </div>
      </div>
    </div>
  </div>

</template>

<script>
import WireGuardHelper from "../../js/wg-helper.js";
import FastEqual from "fast-deep-equal";


export default {
  name: "scripts-island",
  props: {
    peer: {
      type: Object,
      default: {},
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
  },
  data() {
    return {
      peer_local: {
        scripts: {
          pre_up: {enabled: false, value: ""},
          post_up: {enabled: false, value: ""},
          pre_down: {enabled: false, value: ""},
          post_down: {enabled: false, value: ""}
        }
      },
      island_change_sum: {
        changed_fields: {scripts: {}},
        errors: {scripts: {}},
      },
      SCRIPTS_KEY_LOOKUP: {
        pre_up: 'PreUp',
        post_up: 'PostUp',
        pre_down: 'PreDown',
        post_down: 'PostDown',
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_script_field: {pre_up: 0, post_up: 0, pre_down: 0, post_down: 0},
      color_div: 'bg-green-50',
    };
  },
  created() {
    this.peer_local.scripts = JSON.parse(JSON.stringify(this.peer.scripts));
  },
  emits: ['updated-change-sum'],
  methods: {
    check_scripts_field_status(field_name) {
      if (FastEqual(this.peer_local.scripts[field_name], this.peer.scripts[field_name])) return [0, ''];
      const ret = WireGuardHelper.checkField('script', this.peer_local.scripts[field_name]);
      if (!ret.status) return [-1, ret.msg];
      return [1, ''];
    },
    emit_island_change_sum() {
      let island_change_sum = JSON.parse(JSON.stringify(this.island_change_sum));
      for (let field in island_change_sum.errors.scripts) {
        if (island_change_sum.errors.scripts[field] === null) delete island_change_sum.errors.scripts[field];
      }
      for (let field in island_change_sum.changed_fields.scripts) {
        if (island_change_sum.changed_fields.scripts[field] === null) delete island_change_sum.changed_fields.scripts[field];
      }
      if (Object.keys(island_change_sum.errors.scripts).length === 0) island_change_sum.errors.scripts = null;
      if (Object.keys(island_change_sum.changed_fields.scripts).length === 0) island_change_sum.changed_fields.scripts = null;
      this.$emit("updated-change-sum", island_change_sum);
    }
  },
  watch: {
    peer_local: {
      handler() {
        let errorDetected = false;
        let changeDetected = false;
        for (let field in this.peer_local.scripts) {
          let msg = "";
          [this.is_changed_script_field[field], msg] = this.check_scripts_field_status(field);
          this.island_change_sum.errors.scripts[field] = this.is_changed_script_field[field] === -1 ? msg : null;
          this.island_change_sum.changed_fields.scripts[field] = this.is_changed_script_field[field] === 1 ? this.peer_local.scripts[field] : null;

          errorDetected ||= this.is_changed_script_field[field] === -1;
          changeDetected ||= this.is_changed_script_field[field] !== 0;
        }
        this.emit_island_change_sum();
        this.color_div = errorDetected ? 'bg-red-50' : changeDetected ? 'bg-green-100' : 'bg-green-50';
      },
      deep: true,
    }
  },
}
</script>

<style scoped>

</style>