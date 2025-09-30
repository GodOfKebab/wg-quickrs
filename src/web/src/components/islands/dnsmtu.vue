<template>
  <div :class="[color_div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
      <!-- DNS -->
      <input-field v-model="peer_local.dns"
                   :placeholder="defaultDnsmtu.dns.value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                   :input-color="FIELD_COLOR_LOOKUP[is_changed_field.dns]"
                   :is-enabled-value="true"
                   :value-prev="peer.dns"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="DNS"></input-field>
      <datalist id="DNS-list">
        <option :value="defaultDnsmtu.dns.value">
          Forward all DNS related traffic to {{ defaultDnsmtu.dns.value }}
        </option>
      </datalist>

      <!-- MTU -->
      <!-- TODO: fix the undo button shadow -->
      <input-field v-model="peer_local.mtu"
                   :placeholder="defaultDnsmtu.mtu.value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                   :input-color="FIELD_COLOR_LOOKUP[is_changed_field.mtu]"
                   :is-enabled-value="true"
                   :value-prev="peer.mtu"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="MTU"></input-field>
      <datalist id="MTU-list">
        <option :value="`${defaultDnsmtu.mtu.value}`">
          Set MTU to {{ defaultDnsmtu.mtu.value }}
        </option>
      </datalist>
    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/js/wg-helper.js";
import FastEqual from "fast-deep-equal";
import InputField from "@/components/ui/input-field.vue";


export default {
  name: "dnsmtu-island",
  components: {InputField},
  props: {
    peer: {
      type: Object,
      default: {},
    },
    defaultDnsmtu: {
      type: Object,
      default: {
        dns: {enabled: false, value: ""},
        mtu: {enabled: false, value: ""},
      },
    },
  },
  data() {
    return {
      peer_local: {dns: {enabled: false, value: ""}, mtu: {enabled: false, value: ""}},
      island_change_sum: {
        changed_fields: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_field: {dns: 0, mtu: 0},
      color_div: 'bg-green-50',
    };
  },
  created() {
    this.peer_local.dns = JSON.parse(JSON.stringify(this.peer.dns));
    this.peer_local.mtu = JSON.parse(JSON.stringify(this.peer.mtu));
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