<template>
  <div :class="[color_div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <div class="grid grid-cols-3 gap-2">
      <!--  Kind  -->
      <input-field v-model="peer_local.kind" :input-color="FIELD_COLOR_LOOKUP[is_changed_field.kind]"
                   :value-prev="peer.kind"
                   label="Kind"
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
      <input-field v-model="peer_local.icon" :input-color="FIELD_COLOR_LOOKUP[is_changed_field.icon]"
                   :is-enabled-value="true"
                   :value-prev="peer.icon"
                   class="col-span-2"
                   label="Icon"
                   placeholder="(e.g. data:image/png;base64,iVBOR...)"></input-field>
    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/js/wg-helper.js";
import FastEqual from "fast-deep-equal";
import InputField from "@/components/ui/input-field.vue";


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
        icon: {enabled: false, value: ""},
      },
    },
  },
  data() {
    return {
      peer_local: {kind: "", icon: {enabled: false, value: ""}},
      island_change_sum: {
        changed_fields: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_field: {kind: 0, icon: 0},
      color_div: 'bg-green-50',
    };
  },
  created() {
    this.peer_local.kind = JSON.parse(JSON.stringify(this.peer.kind));
    this.peer_local.icon = JSON.parse(JSON.stringify(this.peer.icon));
  },
  emits: ['updated-change-sum'],
  methods: {
    _fast_equal(s1, s2) {
      return FastEqual(s1, s2);
    },
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