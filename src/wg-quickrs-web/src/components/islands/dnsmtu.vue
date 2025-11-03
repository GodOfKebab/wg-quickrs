<template>
  <div :class="[colors.div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
      <!-- DNS -->
      <input-field v-model="peer_local_str.dns"
                   :placeholder="defaultDnsmtu.dns.value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                   :input-color="colors.dns"
                   value-field="addresses"
                   :value-prev="peer_str.dns"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="DNS"></input-field>
      <datalist id="DNS-list">
        <option :value="stringify_dns_addresses(defaultDnsmtu.dns.addresses)">
          Forward all DNS related traffic to {{ stringify_dns_addresses(defaultDnsmtu.dns.addresses) }}
        </option>
      </datalist>

      <!-- MTU -->
      <input-field v-model="peer_local_str.mtu"
                   :placeholder="defaultDnsmtu.mtu.value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                   :input-color="colors.mtu"
                   value-field="value"
                   :value-prev="peer_str.mtu"
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
import WireGuardHelper from "@/src/js/wg-helper.js";
import InputField from "@/src/components/ui/input-field.vue";
import {
  validate_peer_dns_wasm,
  validate_peer_mtu_wasm,
} from '@/pkg/wg_quickrs_lib.js'


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
        dns: {enabled: false, value: []},
        mtu: {enabled: false, value: 0},
      },
    },
    isNewPeer: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      peer_str: {dns: {enabled: false, addresses: ""}, mtu: {enabled: false, value: ""}},
      peer_local_str: {dns: {enabled: false, addresses: ""}, mtu: {enabled: false, value: ""}},
      FIELD_COLOR_LOOKUP: null,
      DIV_COLOR_LOOKUP: null,
      colors: {dns: null, mtu: null, div: null},
    };
  },
  created() {
    this.peer_local_str.dns.enabled = this.peer.dns.enabled;
    this.peer_local_str.dns.addresses = this.stringify_dns_addresses(this.peer.dns.addresses);
    this.peer_local_str.mtu.enabled = this.peer.mtu.enabled;
    this.peer_local_str.mtu.value = this.peer.mtu.value.toString();
    this.peer_str = JSON.parse(JSON.stringify(this.peer_local_str));
    this.FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(this.isNewPeer);
    this.DIV_COLOR_LOOKUP = WireGuardHelper.get_div_colors(this.isNewPeer);
  },
  emits: ['updated-change-sum'],
  methods: {
    stringify_dns_addresses(addresses) {
      return addresses.join(", ");
    }
  },
  watch: {
    peer_local_str: {
      handler() {
        // Initialize the change sum object
        let island_change_sum = {
          errors: {},
          changed_fields: {}
        };

        // dns
        [this.colors.dns, island_change_sum] = WireGuardHelper.validateField(
            'dns',
            validate_peer_dns_wasm,
            this.peer.dns,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.dns.enabled,   // validator args
            this.peer_local_str.dns.addresses  // validator args
        );

        // mtu
        [this.colors.mtu, island_change_sum] = WireGuardHelper.validateField(
            'mtu',
            validate_peer_mtu_wasm,
            this.peer.mtu,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.mtu.enabled,  // validator args
            this.peer_local_str.mtu.value     // validator args
        );

        // Check for errors or changes
        const errorDetected = Object.values(island_change_sum.errors).some(err => err !== null);
        const changeDetected = Object.values(island_change_sum.changed_fields).some(field => field !== null);
        this.colors.div = errorDetected ? this.DIV_COLOR_LOOKUP.error : changeDetected ? this.DIV_COLOR_LOOKUP.changed : this.DIV_COLOR_LOOKUP.unchanged;

        this.$emit("updated-change-sum", island_change_sum);
      },
      deep: true,
    }
  },
}
</script>

<style scoped>

</style>