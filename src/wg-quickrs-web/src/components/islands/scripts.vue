<template>
  <div :class="[color_div]" class="py-2 pl-1 pr-3 shadow-md border rounded">
    <!-- Add buttons -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-2 pl-2 pb-1">
      <div v-for="field in Object.keys(SCRIPTS_KEY_LOOKUP)" :key="field" class="items-center justify-center pt-1 border-gray-100">
        <button class="text-gray-700 border-2 border-gray-500 py-2 px-1 rounded items-center transition w-full enabled:hover:bg-green-700 enabled:hover:border-green-700 disabled:bg-gray-400 disabled:border-gray-400 enabled:hover:text-white"
                @click="peer_local_scripts[field].push({enabled: true, value: ''})">
          <span class="text-base inline-block whitespace-pre">+ Add a </span>
          <span class="text-base inline-block"><strong>{{ SCRIPTS_KEY_LOOKUP[field] }}</strong> Script</span>
        </button>
      </div>
    </div>

    <!-- enabled value lists -->
    <div v-for="field in Object.keys(SCRIPTS_KEY_LOOKUP)" :key="field">
      <div v-for="i in peer_local_scripts[field].length" :key="i" class="flex">
        <div class="inline-block my-auto flex-none pl-2">
          <delete-button title="Delete this script"
                         :disabled="peer_local_scripts.deleted[field].has(i-1)"
                         image-classes="h-6 w-6"
                         @click="peer_local_scripts.deleted[field].add(i-1); peer_local_scripts[field][i-1] = peer.scripts[field][i-1] ? peer.scripts[field][i-1] : { enabled: true, value: ''}"></delete-button>
        </div>
        <div class="inline-block flex-1 relative">
          <input-field v-model="peer_local_scripts[field][i-1]"
                       :class="peer_local_scripts.deleted[field].has(i-1) ? 'opacity-50' : ''"
                       :disabled="peer_local_scripts.deleted[field].has(i-1)"
                       :input-color="FIELD_COLOR_LOOKUP[is_changed_script_field[field][i-1]]"
                       :is-enabled-value="true"
                       :value-prev="peer.scripts[field][i-1] ? peer.scripts[field][i-1] : { enabled: true, value: ''}"
                       undo-button-alignment-classes="right-[5px] top-[6px]"
                       :label="SCRIPTS_KEY_LOOKUP[field]"
                       :placeholder="`${SCRIPTS_KEY_LOOKUP[field]} Script (e.g. echo 'Hey, this is ${SCRIPTS_KEY_LOOKUP[field]} Script';)`"></input-field>
          <!-- Undo Button -->
          <undo-button v-if="peer_local_scripts.deleted[field].has(i-1)"
                       :disabled="!peer_local_scripts.deleted[field].has(i-1)"
                       alignment-classes="right-[6px] top-[5px] bg-gray-200"
                       image-classes="h-7"
                       class="rounded"
                       @click="peer_local_scripts.deleted[field].delete(i-1)">
          </undo-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/js/wg-helper.js";
import FastEqual from "fast-deep-equal";
import InputField from "@/components/ui/input-field.vue";
import UndoButton from "@/components/ui/buttons/undo.vue";
import DeleteButton from "@/components/ui/buttons/delete.vue";


export default {
  name: "scripts-island",
  components: {DeleteButton, UndoButton, InputField},
  props: {
    peer: {
      type: Object,
      default: {},
    },
  },
  data() {
    return {
      peer_local_scripts: {
        pre_up: [],
        post_up: [],
        pre_down: [],
        post_down: [],
        deleted: { // to be initialized in created()
          pre_up: null,
          post_up: null,
          pre_down: null,
          post_down: null,
        }
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
  beforeMount() {
    const peer_local_scripts = JSON.parse(JSON.stringify(this.peer.scripts));
    this.peer_local_scripts.pre_up = peer_local_scripts.pre_up;
    this.peer_local_scripts.post_up = peer_local_scripts.post_up;
    this.peer_local_scripts.pre_down = peer_local_scripts.pre_down;
    this.peer_local_scripts.post_down = peer_local_scripts.post_down;
    this.peer_local_scripts.deleted.pre_up = new Set();
    this.peer_local_scripts.deleted.post_up = new Set();
    this.peer_local_scripts.deleted.pre_down = new Set();
    this.peer_local_scripts.deleted.post_down = new Set();
  },
  emits: ['updated-change-sum'],
  methods: {
    check_scripts_field_status(field_name, i) {
      if (FastEqual(this.peer_local_scripts[field_name][i], this.peer.scripts[field_name][i])) return [0, ''];
      const ret = WireGuardHelper.checkField('script', this.peer_local_scripts[field_name][i]);
      if (!ret.status) return [-1, ret.msg];
      return [1, ''];
    },
    emit_island_change_sum(island_change_sum) {
      for (let field in island_change_sum.errors.scripts) {
        if (island_change_sum.errors.scripts[field] === null) delete island_change_sum.errors.scripts[field];
      }
      for (let field in island_change_sum.changed_fields.scripts) {
        if (island_change_sum.changed_fields.scripts[field] === null) delete island_change_sum.changed_fields.scripts[field];
      }
      this.$emit("updated-change-sum", island_change_sum);
    }
  },
  watch: {
    peer_local_scripts: {
      handler() {
        const island_change_sum = {
          changed_fields: {
            scripts: {
              pre_up: null,
              post_up: null,
              pre_down: null,
              post_down: null,
            },
          },
          errors: {
            scripts: {
              pre_up: null,
              post_up: null,
              pre_down: null,
              post_down: null,
            }
          }
        }
        let errorDetected = false;
        let changeDetected = false;
        const is_changed_script_field = {}
        for (const field of Object.keys(this.SCRIPTS_KEY_LOOKUP)) {
          let errorDetectedField = false;
          let changeDetectedField = false;
          is_changed_script_field[field] = [];
          for (let i = 0; i < this.peer_local_scripts[field].length; i++) {
            if (this.peer_local_scripts.deleted[field].has(i)) continue;
            let msg = "";
            is_changed_script_field[field].push(0);
            [is_changed_script_field[field][i], msg] = this.check_scripts_field_status(field, i);
            island_change_sum.errors.scripts[field] = is_changed_script_field[field][i] === -1 ? msg : island_change_sum.errors.scripts[field];
            island_change_sum.changed_fields.scripts[field] = is_changed_script_field[field][i] === 1 ? this.peer_local_scripts[field] : island_change_sum.changed_fields.scripts[field];

            errorDetectedField ||= is_changed_script_field[field][i] === -1;
            changeDetectedField ||= is_changed_script_field[field][i] !== 0;
          }

          if (this.peer_local_scripts[field].length !== this.peer.scripts[field].length && changeDetectedField && !errorDetectedField) {
            island_change_sum.changed_fields.scripts[field] = this.peer_local_scripts[field];
          }
          if (this.peer_local_scripts.deleted[field].size > 0) {
            island_change_sum.changed_fields.scripts[field] = JSON.parse(JSON.stringify(this.peer_local_scripts[field]));

            let sorted_deleted_indices = Array.from(this.peer_local_scripts.deleted[field]);
            sorted_deleted_indices.sort((a, b) => a - b);
            for (const [_i, i] of sorted_deleted_indices.entries()) {
              island_change_sum.changed_fields.scripts[field].splice(i-_i, 1);
            }
            if (FastEqual(island_change_sum.changed_fields.scripts[field], this.peer.scripts[field])) {
              island_change_sum.changed_fields.scripts[field] = null;
            }
          }

          errorDetected ||= errorDetectedField;
          changeDetected ||= changeDetectedField;
        }
        this.is_changed_script_field = is_changed_script_field;
        this.emit_island_change_sum(island_change_sum);
        this.color_div = errorDetected ? 'bg-red-50' : changeDetected ? 'bg-green-100' : 'bg-green-50';
      },
      deep: true,
    }
  },
}
</script>

<style scoped>

</style>