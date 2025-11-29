<template>
  <div :class="[colors.div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <div class="grid grid-cols-1 gap-2">
      <!-- Persistent Keepalive -->
      <input-field v-model="connection_local_str.persistent_keepalive"
                   :placeholder="defaultPersistentKeepalive.period ? 'Click to see recommendations' : 'No recommendations'"
                   :input-color="colors.persistent_keepalive"
                   value-field="period"
                   :value-prev="connection_str.persistent_keepalive"
                   undo-button-alignment-classes="right-[5px] top-[6px]"
                   label="PersistentKeepalive"></input-field>
      <datalist id="PersistentKeepalive-list">
        <option :value="`${defaultPersistentKeepalive.period}`">
          Set PersistentKeepalive to {{ defaultPersistentKeepalive.period }} seconds
        </option>
      </datalist>
    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/src/js/wg-helper.js";
import InputField from "@/src/components/ui/input-field.vue";
import {
  validate_conn_persistent_keepalive_wasm,
} from '@/pkg/wg_quickrs_lib.js'


export default {
  name: "persistent-keepalive-island",
  components: {InputField},
  props: {
    connection: {
      type: Object,
      default: {},
    },
    defaultPersistentKeepalive: {
      type: Object,
      default: {
        enabled: false,
        period: 0,
      },
    },
    isNewConnection: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      connection_str: {persistent_keepalive: {enabled: false, period: ""}},
      connection_local_str: {persistent_keepalive: {enabled: false, period: ""}},
      FIELD_COLOR_LOOKUP: null,
      DIV_COLOR_LOOKUP: null,
      colors: {persistent_keepalive: null, div: null},
    };
  },
  created() {
    this.connection_local_str.persistent_keepalive.enabled = this.connection.persistent_keepalive.enabled;
    this.connection_local_str.persistent_keepalive.period = this.connection.persistent_keepalive.period.toString();
    this.connection_str = JSON.parse(JSON.stringify(this.connection_local_str));
    this.FIELD_COLOR_LOOKUP = WireGuardHelper.get_field_colors(this.isNewConnection);
    this.DIV_COLOR_LOOKUP = WireGuardHelper.get_div_colors(this.isNewConnection);
  },
  emits: ['updated-change-sum'],
  methods: {},
  watch: {
    connection_local_str: {
      handler() {
        // Initialize the change sum object
        let island_change_sum = {
          errors: {},
          changed_fields: {}
        };

        // persistent_keepalive
        [this.colors.persistent_keepalive, island_change_sum] = WireGuardHelper.validateField(
            'persistent_keepalive',
            validate_conn_persistent_keepalive_wasm,
            this.connection.persistent_keepalive,
            island_change_sum,
            this.FIELD_COLOR_LOOKUP,
            this.connection_local_str.persistent_keepalive.enabled,   // validator args
            this.connection_local_str.persistent_keepalive.period     // validator args
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
