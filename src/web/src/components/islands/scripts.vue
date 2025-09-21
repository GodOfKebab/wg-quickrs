<template>

  <div :class="[color_div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <!-- PreUp -->
    <input-field v-model="peer_local_scripts.pre_up"
                 :input-color="FIELD_COLOR_LOOKUP[is_changed_script_field.pre_up]"
                 :is-enabled-value="true"
                 :value-prev="peer.scripts.pre_up"
                 field-label="PreUp"
                 field-placeholder="PreUp Script (e.g. echo 'Hey, this is PreUp Script';)"></input-field>

    <!-- PostUp -->
    <input-field v-model="peer_local_scripts.post_up"
                 :input-color="FIELD_COLOR_LOOKUP[is_changed_script_field.post_up]"
                 :is-enabled-value="true"
                 :value-prev="peer.scripts.post_up"
                 field-label="PostUp"
                 field-placeholder="PostUp Script (e.g. echo 'Hey, this is PostUp Script';)"></input-field>

    <!-- PreDown -->
    <input-field v-model="peer_local_scripts.pre_down"
                 :input-color="FIELD_COLOR_LOOKUP[is_changed_script_field.pre_down]"
                 :is-enabled-value="true"
                 :value-prev="peer.scripts.pre_down"
                 field-label="PreDown"
                 field-placeholder="PreDown Script (e.g. echo 'Hey, this is PreDown Script';)"></input-field>

    <!-- PostDown -->
    <input-field v-model="peer_local_scripts.post_down"
                 :input-color="FIELD_COLOR_LOOKUP[is_changed_script_field.post_down]"
                 :is-enabled-value="true"
                 :value-prev="peer.scripts.post_down"
                 field-label="PostDown"
                 field-placeholder="PostDown Script (e.g. echo 'Hey, this is PostDown Script';)"></input-field>
  </div>

</template>

<script>
import WireGuardHelper from "@/js/wg-helper.js";
import FastEqual from "fast-deep-equal";
import InputField from "@/components/ui/input-field.vue";


export default {
  name: "scripts-island",
  components: {InputField},
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
      peer_local_scripts: {
        pre_up: {enabled: false, value: ""},
        post_up: {enabled: false, value: ""},
        pre_down: {enabled: false, value: ""},
        post_down: {enabled: false, value: ""}
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
    this.peer_local_scripts = JSON.parse(JSON.stringify(this.peer.scripts));
  },
  emits: ['updated-change-sum'],
  methods: {
    check_scripts_field_status(field_name) {
      if (FastEqual(this.peer_local_scripts[field_name], this.peer.scripts[field_name])) return [0, ''];
      const ret = WireGuardHelper.checkField('script', this.peer_local_scripts[field_name]);
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
      this.$emit("updated-change-sum", island_change_sum);
    }
  },
  watch: {
    peer_local_scripts: {
      handler() {
        let errorDetected = false;
        let changeDetected = false;
        for (let field in this.peer_local_scripts) {
          let msg = "";
          [this.is_changed_script_field[field], msg] = this.check_scripts_field_status(field);
          this.island_change_sum.errors.scripts[field] = this.is_changed_script_field[field] === -1 ? msg : null;
          this.island_change_sum.changed_fields.scripts[field] = this.is_changed_script_field[field] === 1 ? this.peer_local_scripts[field] : null;

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