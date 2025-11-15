<template>
  <div :class="[colors.div]" class="py-2 pl-1 pr-3 shadow-md border rounded">
    <!-- Add buttons -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-2 pl-2 pb-1">
      <div v-for="field in Object.keys(SCRIPTS_KEY_LOOKUP)" :key="field" class="items-center justify-center pt-1 border-gray-100">
        <button class="text-gray-700 border-2 border-gray-500 py-2 px-1 rounded items-center transition w-full enabled:hover:bg-green-700 enabled:hover:border-green-700 disabled:bg-gray-400 disabled:border-gray-400 enabled:hover:text-gray-100"
                @click="peer_local_scripts[field].push({enabled: true, script: ''})">
          <span class="text-base inline-block whitespace-pre">+ Add a </span>
          <span class="text-base inline-block"><strong>{{ SCRIPTS_KEY_LOOKUP[field] }}</strong> Script</span>
        </button>
      </div>
    </div>

    <!-- scripts -->
    <div v-for="field in Object.keys(SCRIPTS_KEY_LOOKUP)" :key="field">
      <div v-for="i in peer_local_scripts[field].length" :key="i" class="flex">
        <div class="inline-block my-auto flex-none pl-2">
          <delete-button title="Delete this script"
                         :disabled="peer_local_scripts.deleted[field].has(i-1)"
                         image-classes="h-6 w-6"
                         @click="peer_local_scripts.deleted[field].add(i-1); peer_local_scripts[field][i-1] = peer.scripts[field][i-1] ? peer.scripts[field][i-1] : { enabled: true, script: ''}"></delete-button>
        </div>
        <div class="inline-block flex-1 relative">
          <input-field v-model="peer_local_scripts[field][i-1]"
                       :class="peer_local_scripts.deleted[field].has(i-1) ? 'opacity-50' : ''"
                       :disabled="peer_local_scripts.deleted[field].has(i-1)"
                       :input-color="colors[field][i-1]"
                       value-field="script"
                       :value-prev="peer.scripts[field][i-1] ? peer.scripts[field][i-1] : { enabled: true, script: ''}"
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
import WireGuardHelper from "@/src/js/wg-helper.js";
import FastEqual from "fast-deep-equal";
import InputField from "@/src/components/ui/input-field.vue";
import UndoButton from "@/src/components/ui/buttons/undo.vue";
import DeleteButton from "@/src/components/ui/buttons/delete.vue";
import {
  validate_peer_script_wasm,
} from '@/pkg/wg_quickrs_lib.js'


export default {
  name: "scripts-island",
  components: {DeleteButton, UndoButton, InputField},
  props: {
    peer: {
      type: Object,
      default: {},
    },
    isNewPeer: {
      type: Boolean,
      default: false,
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
          pre_up: [],
          post_up: [],
          pre_down: [],
          post_down: [],
        }
      },
      SCRIPTS_KEY_LOOKUP: {
        pre_up: 'PreUp',
        post_up: 'PostUp',
        pre_down: 'PreDown',
        post_down: 'PostDown',
      },
      EXISTING_FIELD_COLOR_LOOKUP: null,
      NEW_FIELD_COLOR_LOOKUP: null,
      DIV_COLOR_LOOKUP: null,
      colors: {pre_up: [], post_up: [], pre_down: [], post_down: [], div: null},
    };
  },
  created() {
    const peer_local_scripts = JSON.parse(JSON.stringify(this.peer.scripts));
    this.peer_local_scripts.pre_up = peer_local_scripts.pre_up;
    this.peer_local_scripts.post_up = peer_local_scripts.post_up;
    this.peer_local_scripts.pre_down = peer_local_scripts.pre_down;
    this.peer_local_scripts.post_down = peer_local_scripts.post_down;
    this.peer_local_scripts.deleted.pre_up = new Set();
    this.peer_local_scripts.deleted.post_up = new Set();
    this.peer_local_scripts.deleted.pre_down = new Set();
    this.peer_local_scripts.deleted.post_down = new Set();
    this.EXISTING_FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(this.isNewPeer);
    this.NEW_FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(true);
    this.DIV_COLOR_LOOKUP = WireGuardHelper.get_div_colors(this.isNewPeer);
  },
  emits: ['updated-change-sum'],
  methods: {
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
              pre_up: [],
              post_up: [],
              pre_down: [],
              post_down: [],
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
        };

        let errorDetected = false;
        let changeDetected = false;

        // Iterate over each script field
        for (const field of Object.keys(this.SCRIPTS_KEY_LOOKUP)) {
          let errorDetectedField = false;
          let changeDetectedField = false;

          // Validate scripts
          for (let i = 0; i < this.peer_local_scripts[field].length; i++) {
            const is_new_script = i >= this.peer.scripts[field].length;
            if (this.peer_local_scripts.deleted[field].has(i)) {
              if (!is_new_script) {
                changeDetectedField = true;
              }
              continue;
            }

            let script_change_sum = {errors: {}, changed_fields: {}};

            const FIELD_COLOR_LOOKUP = is_new_script ? this.NEW_FIELD_COLOR_LOOKUP : this.EXISTING_FIELD_COLOR_LOOKUP;

            [this.colors[field][i], script_change_sum] = WireGuardHelper.validateField(
                field,
                validate_peer_script_wasm,
                this.peer.scripts[field][i] || null,
                script_change_sum,
                FIELD_COLOR_LOOKUP,
                this.peer_local_scripts[field][i].enabled,
                this.peer_local_scripts[field][i].script
            );

            // Update island change sum
            if (script_change_sum.errors[field]) {
              island_change_sum.errors.scripts[field] = script_change_sum.errors[field];
              errorDetectedField = true;
            } else if (script_change_sum.changed_fields[field]) {
              island_change_sum.changed_fields.scripts[field].push(this.peer_local_scripts[field][i]);
              changeDetectedField = true;
            } else {
              island_change_sum.changed_fields.scripts[field].push(this.peer.scripts[field][i]);
            }
          }
          if (!changeDetectedField) {
            island_change_sum.changed_fields.scripts[field] = null;
          }

          errorDetected ||= errorDetectedField;
          changeDetected ||= changeDetectedField;
        }

        this.emit_island_change_sum(island_change_sum);
        this.colors.div = errorDetected ? this.DIV_COLOR_LOOKUP.error : changeDetected ? this.DIV_COLOR_LOOKUP.changed : this.DIV_COLOR_LOOKUP.unchanged;
      },
      deep: true,
    }
  },
}
</script>

<style scoped>

</style>