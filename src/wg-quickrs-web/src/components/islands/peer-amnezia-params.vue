<template>
  <div :class="[colors.div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <div class="grid grid-cols-1">

      <!-- Jc Parameter -->
      <input-field v-model="peer_local_str.amnezia_parameters.jc"
                   :input-color="colors.jc"
                   :value-prev="peer_str.amnezia_parameters.jc"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="Jc (Junk Packet Count)"
                   placeholder="-1 to 128"></input-field>

      <!-- Jmin Parameter -->
      <input-field v-model="peer_local_str.amnezia_parameters.jmin"
                   :input-color="colors.jmin"
                   :value-prev="peer_str.amnezia_parameters.jmin"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="Jmin (Min Junk Packet Size)"
                   placeholder="1-1279"></input-field>

      <!-- Jmax Parameter -->
      <input-field v-model="peer_local_str.amnezia_parameters.jmax"
                   :input-color="colors.jmax"
                   :value-prev="peer_str.amnezia_parameters.jmax"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="Jmax (Max Junk Packet Size)"
                   placeholder="1-1280, jmin<jmax"></input-field>

    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/src/js/wg-helper.js";
import InputField from "@/src/components/ui/input-field.vue";
import {
  validate_amnezia_jc_wasm,
  validate_amnezia_jmin_wasm,
  validate_amnezia_jmin_jmax_wasm,
} from '@/pkg/wg_quickrs_lib.js'

export default {
  name: "peer-amnezia-params-island",
  components: {InputField},
  props: {
    peer: {
      type: Object,
      default: {},
    },
    defaultAmneziaParameters: {
      type: Object,
      default: {
        jc: 0,
        jmin: 0,
        jmax: 0,
      },
    },
    isNewPeer: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      peer_str: {amnezia_parameters: {jc: "", jmin: "", jmax: ""}},
      peer_local_str: {amnezia_parameters: {jc: "", jmin: "", jmax: ""}},
      FIELD_COLOR_LOOKUP: null,
      DIV_COLOR_LOOKUP: null,
      colors: {jc: null, jmin: null, jmax: null, div: null},
    };
  },
  created() {
    this.peer_local_str.amnezia_parameters.jc = this.peer.amnezia_parameters.jc.toString();
    this.peer_local_str.amnezia_parameters.jmin = this.peer.amnezia_parameters.jmin.toString();
    this.peer_local_str.amnezia_parameters.jmax = this.peer.amnezia_parameters.jmax.toString();
    this.peer_str = JSON.parse(JSON.stringify(this.peer_local_str));

    this.FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(this.isNewPeer);
    this.DIV_COLOR_LOOKUP = WireGuardHelper.get_div_colors(this.isNewPeer);
  },
  emits: ['updated-change-sum'],
  methods: {},
  watch: {
    peer_local_str: {
      handler() {
        // Initialize the change sum object
        let island_change_sum = {
          errors: {},
          changed_fields: {}
        };

        [this.colors.jc, island_change_sum] = WireGuardHelper.validateField(
            'jc',
            validate_amnezia_jc_wasm,
            this.peer.amnezia_parameters.jc,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.amnezia_parameters.jc
        );

        [this.colors.jmin, island_change_sum] = WireGuardHelper.validateField(
            'jmin',
            validate_amnezia_jmin_wasm,
            this.peer.amnezia_parameters.jmin,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.amnezia_parameters.jmin
        );

        [this.colors.jmax, island_change_sum] = WireGuardHelper.validateField(
            'jmax',
            validate_amnezia_jmin_jmax_wasm,
            this.peer.amnezia_parameters.jmax,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local_str.amnezia_parameters.jmin,
            this.peer_local_str.amnezia_parameters.jmax
        );

        // Check for errors or changes
        const errorDetected = Object.values(island_change_sum.errors).some(err => err !== null);
        const changeDetected = Object.values(island_change_sum.changed_fields).some(field => field !== null);
        this.colors.div = errorDetected ? this.DIV_COLOR_LOOKUP.error : changeDetected ? this.DIV_COLOR_LOOKUP.changed : this.DIV_COLOR_LOOKUP.unchanged;

        this.$emit("updated-change-sum", {
          errors: {
            amnezia_parameters: island_change_sum.errors,
          },
          changed_fields: {
            amnezia_parameters: island_change_sum.changed_fields,
          }
        });
      },
      deep: true,
    }
  },
}
</script>

<style scoped>

</style>
