<template>
  <div :class="[colors.div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <div class="grid grid-cols-2 md:grid-cols-3 gap-2">
      <!--  Kind  -->
      <input-field v-model="peer_local.kind" :input-color="colors.kind"
                   :value-prev="peer.kind"
                   class="col-span-2 md:col-span-1"
                   label="Kind"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   placeholder="Kind"></input-field>
      <datalist id="Kind-list">
        <option value="server"></option>
        <option value="desktop"></option>
        <option value="laptop"></option>
        <option value="tablet"></option>
        <option value="phone"></option>
        <option value="IoT"></option>
        <option value="other"></option>
      </datalist>

      <!-- Icon -->
      <input-field v-model="peer_local.icon" :input-color="colors.icon"
                   value-field="src"
                   :value-prev="peer.icon"
                   class="col-span-2"
                   label="Icon"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   placeholder="(e.g. data:image/png;base64,iVBOR...)"></input-field>
    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/src/js/wg-helper.js";
import InputField from "@/src/components/ui/input-field.vue";
import {
  validate_peer_kind_wasm,
  validate_peer_icon_wasm,
} from '@/pkg/wg_quickrs_lib.js'


export default {
  name: "peer-kind-icon-island",
  components: {InputField},
  props: {
    peer: {
      type: Object,
      default: {},
    },
    defaultKindIcon: {
      type: Object,
      default: {
        kind: "",
        icon: {enabled: false, src: ""},
      },
    },
    isNewPeer: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      peer_local: {kind: "", icon: {enabled: false, src: ""}},
      FIELD_COLOR_LOOKUP: null,
      DIV_COLOR_LOOKUP: null,
      colors: {kind: null, icon: null, div: null},
    };
  },
  created() {
    this.peer_local.kind = JSON.parse(JSON.stringify(this.peer.kind));
    this.peer_local.icon = JSON.parse(JSON.stringify(this.peer.icon));
    this.FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(this.isNewPeer);
    this.DIV_COLOR_LOOKUP = WireGuardHelper.get_div_colors(this.isNewPeer);
  },
  emits: ['updated-change-sum'],
  methods: {},
  watch: {
    peer_local: {
      handler() {
        // Initialize the change sum object
        let island_change_sum = {
          errors: {},
          changed_fields: {}
        };

        // kind
        [this.colors.kind, island_change_sum] = WireGuardHelper.validateField(
            'kind',
            validate_peer_kind_wasm,
            this.peer.kind,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local.kind  // validator arg
        );

        // icon
        [this.colors.icon, island_change_sum] = WireGuardHelper.validateField(
            'icon',
            validate_peer_icon_wasm,
            this.peer.icon,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.peer_local.icon.enabled,  // validator args
            this.peer_local.icon.src       // validator args
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