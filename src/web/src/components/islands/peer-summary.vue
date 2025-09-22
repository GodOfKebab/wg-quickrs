<template>
  <div :class="[color_div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <!--  Name  -->
    <input-field v-model="peer_local.name" :input-color="FIELD_COLOR_LOOKUP[is_changed_field.name]"
                 :value-prev="peer.name"
                 label="Name"
                 placeholder="Name"></input-field>

    <!--  Address  -->
    <!-- TODO: update connection address on change -->
    <input-field v-model="peer_local.address" :input-color="FIELD_COLOR_LOOKUP[is_changed_field.address]"
                 :disabled="isNewPeer"
                 :value-prev="peer.address"
                 label="Address"
                 placeholder="Address (e.g. 10.8.0.1)"></input-field>

    <!--  Endpoint  -->
    <input-field v-model="peer_local.endpoint" :input-color="FIELD_COLOR_LOOKUP[is_changed_field.endpoint]"
                 :is-enabled-value="true"
                 :value-prev="peer.endpoint"
                 label="Static Endpoint"
                 placeholder="Endpoint (e.g. 1.2.3.4:51820 or example.com:51820)"></input-field>
  </div>
</template>

<script>


import WireGuardHelper from "@/js/wg-helper.js";
import FastEqual from "fast-deep-equal";
import InputField from "@/components/ui/input-field.vue";

export default {
  name: "peer-summary",
  components: {InputField, StringField: InputField},
  props: {
    peer: {
      type: Object,
      default: {},
    },
    isHost: {
      type: Boolean,
      default: false,
    },
    isNewPeer: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      peer_local: {name: null, address: null, endpoint: null},
      island_change_sum: {
        changed_fields: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_field: {name: 0, address: 0, endpoint: {enabled: 0, value: 0}},
      color_div: 'bg-green-50',
    };
  },

  created() {
    this.peer_local.name = JSON.parse(JSON.stringify(this.peer.name));
    this.peer_local.address = JSON.parse(JSON.stringify(this.peer.address));
    this.peer_local.endpoint = JSON.parse(JSON.stringify(this.peer.endpoint));
  },
  emits: ['updated-change-sum'],
  methods: {
    check_field_status(field_name) {
      const ret = WireGuardHelper.checkField(field_name, this.peer_local[field_name]);
      if (!ret.status) return [-1, ret.msg];
      if (FastEqual(this.peer_local[field_name], this.peer[field_name])) return [0, ''];
      return [1, ''];
    },
    emit_island_change_sum() {
      this.$emit("updated-change-sum", this.island_change_sum);
    }
  },
  watch: {
    peer_local: {
      handler() {
        let errorDetected = false;
        let changeDetected = false;
        for (let field in this.peer_local) {
          let msg = "";
          [this.is_changed_field[field], msg] = this.check_field_status(field);
          this.island_change_sum.errors[field] = this.is_changed_field[field] === -1 ? msg : null;
          this.island_change_sum.changed_fields[field] = this.is_changed_field[field] === 1 ? this.peer_local[field] : null;

          errorDetected ||= this.is_changed_field[field] === -1;
          changeDetected ||= this.is_changed_field[field] !== 0;
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