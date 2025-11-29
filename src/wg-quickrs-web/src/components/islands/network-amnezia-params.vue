<template>
  <div :class="[colors.div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <h5 class="text-xl mb-3 pl-2">Network-Level Amnezia Parameters</h5>
    <div class="grid grid-cols-1">

      <!-- Enabled Toggle -->
      <div class="flex items-center gap-2">
        <label class="text-sm font-medium">Enable AmneziaWG:</label>
        <input
            v-model="network_local_str.amnezia_parameters.enabled"
            :class="[colors.enabled]"
            type="checkbox"
            class="h-4 w-4">
      </div>

      <!-- S1 Parameter -->
      <input-field v-model="network_local_str.amnezia_parameters.s1"
                   :input-color="colors.s1"
                   :value-prev="network_str.amnezia_parameters.s1"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="S1 (Init Packet Junk Size)"
                   placeholder="0-1132"></input-field>

      <!-- S2 Parameter -->
      <input-field v-model="network_local_str.amnezia_parameters.s2"
                   :input-color="colors.s2"
                   :value-prev="network_str.amnezia_parameters.s2"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="S2 (Response Packet Junk Size)"
                   placeholder="0-1188, s1+56≠s2"></input-field>

      <!-- H1 Parameter -->
      <input-field v-model="network_local_str.amnezia_parameters.h1"
                   :input-color="colors.h1"
                   :value-prev="network_str.amnezia_parameters.h1"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="H1 (Init Packet Magic Header)"
                   placeholder="32-bit unsigned integer"></input-field>

      <!-- H2 Parameter -->
      <input-field v-model="network_local_str.amnezia_parameters.h2"
                   :input-color="colors.h2"
                   :value-prev="network_str.amnezia_parameters.h2"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="H2 (Response Packet Magic Header)"
                   placeholder="32-bit unsigned integer"></input-field>

      <!-- H3 Parameter -->
      <input-field v-model="network_local_str.amnezia_parameters.h3"
                   :input-color="colors.h3"
                   :value-prev="network_str.amnezia_parameters.h3"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="H3 (Underload Packet Magic Header)"
                   placeholder="32-bit unsigned integer"></input-field>

      <!-- H4 Parameter -->
      <input-field v-model="network_local_str.amnezia_parameters.h4"
                   :input-color="colors.h4"
                   :value-prev="network_str.amnezia_parameters.h4"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="H4 (Transport Packet Magic Header)"
                   placeholder="32-bit unsigned integer"></input-field>

    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/src/js/wg-helper.js";
import InputField from "@/src/components/ui/input-field.vue";
import {
  validate_amnezia_s1_wasm,
  validate_amnezia_s1_s2_wasm,
  validate_amnezia_h_wasm,
} from '@/pkg/wg_quickrs_lib.js'

export default {
  name: "network-amnezia-params-island",
  components: {InputField},
  props: {
    network: {
      type: Object,
      default: {},
    },
    isNewNetwork: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      network_str: {amnezia_parameters: {enabled: false, s1: "", s2: "", h1: "", h2: "", h3: "", h4: ""}},
      network_local_str: {amnezia_parameters: {enabled: false, s1: "", s2: "", h1: "", h2: "", h3: "", h4: ""}},
      FIELD_COLOR_LOOKUP: null,
      DIV_COLOR_LOOKUP: null,
      colors: {enabled: null, s1: null, s2: null, h1: null, h2: null, h3: null, h4: null, div: null},
    };
  },
  created() {
    this.network_local_str.amnezia_parameters.enabled = this.network.amnezia_parameters.enabled;
    this.network_local_str.amnezia_parameters.s1 = this.network.amnezia_parameters.s1.toString();
    this.network_local_str.amnezia_parameters.s2 = this.network.amnezia_parameters.s2.toString();
    this.network_local_str.amnezia_parameters.h1 = this.network.amnezia_parameters.h1.toString();
    this.network_local_str.amnezia_parameters.h2 = this.network.amnezia_parameters.h2.toString();
    this.network_local_str.amnezia_parameters.h3 = this.network.amnezia_parameters.h3.toString();
    this.network_local_str.amnezia_parameters.h4 = this.network.amnezia_parameters.h4.toString();
    this.network_str = JSON.parse(JSON.stringify(this.network_local_str));

    this.FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(this.isNewNetwork);
    this.DIV_COLOR_LOOKUP = WireGuardHelper.get_div_colors(this.isNewNetwork);
  },
  emits: ['updated-change-sum'],
  methods: {},
  watch: {
    network_local_str: {
      handler() {
        // Initialize the change sum object
        let island_change_sum = {
          errors: {},
          changed_fields: {}
        };

        // Validate enabled field
        if (this.network_local_str.amnezia_parameters.enabled !== this.network.amnezia_parameters.enabled) {
          island_change_sum.changed_fields.amnezia_parameters = island_change_sum.changed_fields.amnezia_parameters || {};
          island_change_sum.changed_fields.amnezia_parameters.enabled = this.network_local_str.amnezia_parameters.enabled;
          this.colors.enabled = this.FIELD_COLOR_LOOKUP.changed;
        } else {
          this.colors.enabled = this.FIELD_COLOR_LOOKUP.unchanged;
        }

        [this.colors.s1, island_change_sum] = WireGuardHelper.validateField(
            's1',
            validate_amnezia_s1_wasm,
            this.network.amnezia_parameters.s1,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.network_local_str.amnezia_parameters.s1
        );

        [this.colors.s2, island_change_sum] = WireGuardHelper.validateField(
            's2',
            validate_amnezia_s1_s2_wasm,
            this.network.amnezia_parameters.s2,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.network_local_str.amnezia_parameters.s1,
            this.network_local_str.amnezia_parameters.s2
        );

        [this.colors.h1, island_change_sum] = WireGuardHelper.validateField(
            'h1',
            validate_amnezia_h_wasm,
            this.network.amnezia_parameters.h1,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.network_local_str.amnezia_parameters.h1
        );

        [this.colors.h2, island_change_sum] = WireGuardHelper.validateField(
            'h2',
            validate_amnezia_h_wasm,
            this.network.amnezia_parameters.h2,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.network_local_str.amnezia_parameters.h2
        );

        [this.colors.h3, island_change_sum] = WireGuardHelper.validateField(
            'h3',
            validate_amnezia_h_wasm,
            this.network.amnezia_parameters.h3,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.network_local_str.amnezia_parameters.h3
        );

        [this.colors.h4, island_change_sum] = WireGuardHelper.validateField(
            'h4',
            validate_amnezia_h_wasm,
            this.network.amnezia_parameters.h4,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.network_local_str.amnezia_parameters.h4
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
