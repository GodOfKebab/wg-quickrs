<template>
  <div :class="[colors.div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <!--  Name  -->
    <input-field v-model="peer_local_str.name" :input-color="colors.name"
                 :value-prev="peer.name"
                 label="Name"
                 undo-button-alignment-classes="right-[5px] top-[6px]"
                 placeholder="Name"></input-field>

    <!--  Address  -->
    <!-- TODO: update connection address on change -->
    <input-field v-model="peer_local_str.address" :input-color="colors.address"
                 :disabled="isNewPeer"
                 :value-prev="peer.address"
                 label="Address"
                 undo-button-alignment-classes="right-[5px] top-[6px]"
                 placeholder="Address (e.g. 10.8.0.1)"></input-field>

    <!--  Endpoint  -->
    <input-field v-model="peer_local_str.endpoint" :input-color="colors.endpoint"
                 value-field="value"
                 :value-prev="{enabled: peer.endpoint.enabled, value: stringify_endpoint(peer.endpoint)}"
                 label="Static Endpoint"
                 undo-button-alignment-classes="right-[5px] top-[6px]"
                 placeholder="Endpoint (e.g. 1.2.3.4:51820 or example.com:51820)"></input-field>
  </div>
</template>

<script>
import WireGuardHelper from "@/src/js/wg-helper.js";
import InputField from "@/src/components/ui/input-field.vue";
import {
  validate_peer_address_wasm,
  validate_peer_endpoint_wasm,
  validate_peer_name_wasm
} from "@/pkg/wg_quickrs_lib.js";

export default {
  name: "peer-summary",
  components: {InputField, StringField: InputField},
  props: {
    network: {
      type: Object,
      default: {},
    },
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
      peer_local_str: {name: "", address: "", endpoint: {enabled: false, value: ""}},
      FIELD_COLOR_LOOKUP: null,
      DIV_COLOR_LOOKUP: null,
      colors: {name: null, address: null, endpoint: null, div: null},
    };
  },

  created() {
    this.peer_local_str.name = this.peer.name;
    this.peer_local_str.address = this.peer.address;
    this.peer_local_str.endpoint.enabled = this.peer.endpoint.enabled;
    this.peer_local_str.endpoint.value = this.stringify_endpoint(this.peer.endpoint);
    this.FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(this.isNewPeer);
    this.DIV_COLOR_LOOKUP = WireGuardHelper.get_div_colors(this.isNewPeer);
  },
  emits: ['updated-change-sum'],
  methods: {
    stringify_endpoint(endpoint) {
      if (endpoint.address === "none") {
        return "";
      }
      if ('ipv4_and_port' in endpoint.address) {
        return `${endpoint.address.ipv4_and_port.ipv4}:${endpoint.address.ipv4_and_port.port}`;
      }
      if ('hostname_and_port' in endpoint.address) {
        return `${endpoint.address.hostname_and_port.hostname}:${endpoint.address.hostname_and_port.port}`;
      }
      return "";
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

        // name
        [this.colors.name, island_change_sum] = WireGuardHelper.validateField(
            'name',
            validate_peer_name_wasm,
            this.peer.name,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.name  // validator arg
        );

        // address
        let network_copy = JSON.parse(JSON.stringify(this.network));
        if (this.isNewPeer) {
          network_copy.reservations = Object.fromEntries(
              Object.entries(this.network.reservations).filter(([key, obj]) => key !== this.peer.address)
          );
        } else {
          network_copy.peers = Object.fromEntries(
              Object.entries(this.network.peers).filter(([key, obj]) => obj.address !== this.peer.address)
          );
        }
        [this.colors.address, island_change_sum] = WireGuardHelper.validateField(
            'address',
            validate_peer_address_wasm,
            this.peer.address,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.address,  // validator arg
            network_copy                  // validator arg
        );

        // endpoint
        [this.colors.endpoint, island_change_sum] = WireGuardHelper.validateField(
            'endpoint',
            validate_peer_endpoint_wasm,
            this.peer.endpoint,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.endpoint.enabled,  // validator args
            this.peer_local_str.endpoint.value     // validator args
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