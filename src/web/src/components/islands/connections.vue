<template>

  <div v-if="other_static_peer_ids.length +  other_roaming_peer_ids.length > 0">
    <!-- selection box -->
    <div :class="[DIV_COLOR_LOOKUP[is_changed_field.attached_peers_box]]"
         class="my-2 py-2 pl-1 pr-3 shadow-md border rounded relative">

      <!-- static neighbors -->
      <div v-if="other_static_peer_ids.length > 0">
        <div class="flex mx-5">
          <div class="text-gray-800 ml-2 text-lg">
            <strong>Attached static peers</strong>:
          </div>

          <div class="form-check mt-1 flex ml-auto">
            <label class="form-check-label flex items-center text-gray-800 cursor-pointer text-sm">
              <input
                  v-model="selectAllStaticPeers"
                  class="h-4 w-4"
                  type="checkbox">
              <span class="align-middle">Select All</span>
            </label>
          </div>
        </div>

        <div class="grid grid-cols-2">
          <div v-for="peerId in other_static_peer_ids"
               class="relative overflow-hidden">
            <div class="form-check mt-1 flex">
              <label class="form-check-label truncate items-center text-gray-800 cursor-pointer text-sm py-1">
                <input
                    v-model="attached_static_peer_ids_local"
                    :value="peerId"
                    class="h-4 w-4"
                    type="checkbox"
                    @change="toggleConnection(peerId)">
                <span class="align-middle">
                           <strong class="text-sm">{{
                               network.peers[peerId].name
                             }}</strong> {{ network.peers[peerId].address }} ({{
                    peerId
                  }})
                         </span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- roaming neighbors -->
      <div v-if=" other_roaming_peer_ids.length > 0">

        <div class="flex mx-5">
          <div class="text-gray-800 ml-2 text-lg">
            <strong>Attached roaming peers</strong>:
          </div>

          <div class="form-check mt-1 flex ml-auto">
            <label class="form-check-label flex items-center text-gray-800 cursor-pointer text-sm">
              <input
                  v-model="selectAllRoamingPeers"
                  class="h-4 w-4"
                  type="checkbox">
              <span class="align-middle">Select All</span>
            </label>
          </div>
        </div>

        <div class="grid grid-cols-2">
          <div v-for="peerId in  other_roaming_peer_ids"
               class="relative overflow-hidden">
            <div class="form-check mt-1 flex">
              <label class="form-check-label truncate items-center text-gray-800 cursor-pointer text-sm py-1">
                <input
                    v-model="attached_roaming_peer_ids_local"
                    :value="peerId"
                    class="h-4 w-4"
                    type="checkbox"
                    @change="toggleConnection(peerId)">
                <span class="align-middle">
                           <strong class="text-sm">{{
                               network.peers[peerId].name
                             }}</strong> {{ network.peers[peerId].address }} ({{
                    peerId
                  }})
                         </span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- undo button -->
      <div v-if="is_changed_field.attached_peers_box"
           class="inline-block float-right absolute z-20 right-[5px] top-[3px]">
        <button
            :disabled="!is_changed_field.attached_peers_box"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
            title="Undo Changes"
            @click="attached_static_peer_ids_local = attached_static_peer_ids; attached_roaming_peer_ids_local = attached_roaming_peer_ids; update_added_removed_change_sum();">
          <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
        </button>
      </div>

    </div>

    <!-- connection islands -->
    <div v-for="otherPeerId in [...other_static_peer_ids, ...other_roaming_peer_ids]"
         class="relative text-sm">
      <div v-if="all_attached_peer_ids_local.includes(otherPeerId)"
           :class="[DIV_COLOR_LOOKUP[is_changed_field.attached_peer_box[otherPeerId]]]"
           class="my-2 py-2 pl-1 pr-3 shadow-md border rounded overflow-x-auto whitespace-nowrap highlight-remove-box">

        <!-- enabled checkbox-->
        <div class="form-check flex">
          <label class="form-check-label items-center text-gray-800 cursor-pointer text-sm">
            <input
                v-model="connections_local.enabled[otherPeerId]"
                class="h-4 w-4"
                type="checkbox">
            <span class="text-gray-800">
                  <strong class="text-sm">{{ network.peers[otherPeerId].name }}</strong>
                  {{ network.peers[otherPeerId].address }}
                  ({{ otherPeerId }})
                </span>
          </label>
        </div>

        <!-- connection details  -->
        <div v-show="connections_local.enabled[otherPeerId]" class="mt-1 mb-0.5 mx-2 text-gray-800">
          <hr class="w-full h-1 mb-1"/>

          <div class="grid grid-cols-7 gap-1">
            <!-- Pre Shared Key -->
            <div v-if="connections_local.pre_shared_key[otherPeerId]" class="col-span-3 flex">
                  <span class="align-middle flex">
                     <strong class="align-middle">PreSharedKey</strong>
                  </span>
            </div>
            <div v-if="connections_local.pre_shared_key[otherPeerId]" class="col-span-4 flex">
                  <span class="pr-1 align-middle">
                    :
                  </span>
              <button
                  class="align-middle rounded bg-gray-100 hover:bg-gray-600 hover:text-white transition-all mr-1 inline-block shrink-0"
                  @click="refreshPreSharedKey(otherPeerId)">
                <img alt="Refresh Keys" class="h-4" src="/icons/flowbite/refresh.svg"/>
              </button>
              <span class="pr-1 align-middle">
                    {{ connections_local.pre_shared_key[otherPeerId] }}
                  </span>
            </div>

            <!-- Persistent Keepalive -->
            <div v-if="connections_local.persistent_keepalive[otherPeerId]" class="col-span-3 flex">
                  <span class="align-middle flex">
                     <strong class="align-middle">Persistent Keepalive</strong>
                  </span>
            </div>
            <div v-if="connections_local.persistent_keepalive[otherPeerId]" class="col-span-4 flex">
              <div class="inline-block align-middle">
                <label class="flex items-center">
                      <span class="pr-1 align-middle">
                        :
                      </span>
                  <input
                      v-model="connections_local.persistent_keepalive[otherPeerId].enabled"
                      class="h-3.5 w-3.5"
                      type="checkbox">
                </label>
              </div>
              <input v-model="connections_local.persistent_keepalive[otherPeerId].value"
                     :class="[FIELD_COLOR_LOOKUP[is_changed_field.persistent_keepalive[otherPeerId]]]"
                     :disabled="!connections_local.persistent_keepalive[otherPeerId].enabled"
                     class="mr-1 rounded-md pl-1 align-middle inline-block disabled:bg-gray-100">
            </div>
          </div>

          <!-- Allowed IPs -->
          <div class="relative text-gray-800">
            <div class="mt-1">
                    <span class="flex-none"><strong>{{
                        network.peers[peerId].name
                      }}</strong> will forward IP subnet(s)</span>
              <input v-if="_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId)"
                     v-model="connections_local.allowed_ips_a_to_b[otherPeerId]"
                     :class="[FIELD_COLOR_LOOKUP[is_changed_field.allowed_ips_a_to_b[otherPeerId]]]"
                     :list="otherPeerId + 'focusPeerName to peerDetails.name'"
                     class="text-gray-800 mx-1 rounded-md px-1 grow">
              <input v-else
                     v-model="connections_local.allowed_ips_b_to_a[otherPeerId]"
                     :class="[FIELD_COLOR_LOOKUP[is_changed_field.allowed_ips_b_to_a[otherPeerId]]]"
                     :list="otherPeerId + 'focusPeerName to peerDetails.name'"
                     class="text-gray-800 mx-1 rounded-md px-1 grow">
              <span class="flex-none pr-2"> to <strong>{{ network.peers[otherPeerId].name }}</strong></span>
            </div>
            <div class="mt-1">
                  <span class="flex-none"><strong>{{
                      network.peers[otherPeerId].name
                    }}</strong> will forward IP subnet(s)</span>
              <input v-if="!_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId)"
                     v-model="connections_local.allowed_ips_a_to_b[otherPeerId]"
                     :class="[FIELD_COLOR_LOOKUP[is_changed_field.allowed_ips_a_to_b[otherPeerId]]]"
                     :list="otherPeerId + 'peerDetails.name to focusPeerName'"
                     class="text-gray-800 mx-1 rounded-md px-1 grow">
              <input v-else
                     v-model="connections_local.allowed_ips_b_to_a[otherPeerId]"
                     :class="[FIELD_COLOR_LOOKUP[is_changed_field.allowed_ips_b_to_a[otherPeerId]]]"
                     :list="otherPeerId + 'peerDetails.name to focusPeerName'"
                     class="text-gray-800 mx-1 rounded-md px-1 grow">
              <span class="flex-none pr-2"> to <strong>{{ network.peers[peerId].name }}</strong></span>
            </div>
            <datalist
                :id="otherPeerId + 'focusPeerName to peerDetails.name'">
              <option value="0.0.0.0/0">
                All traffic
              </option>
              <option :value="network.subnet">
                Only VPN subnet
              </option>
              <option :value="network.peers[otherPeerId].address + '/32'">
                Only {{ network.peers[otherPeerId].name }}
              </option>
            </datalist>
            <datalist
                :id="otherPeerId + 'peerDetails.name to focusPeerName'">
              <option :value="network.peers[peerId].address + '/32'">
                Only {{ network.peers[peerId].name }}
              </option>
            </datalist>
          </div>

        </div>

        <!-- undo button -->
        <div v-if="is_changed_field.attached_peer_box[otherPeerId]"
             class="inline-block float-right absolute z-20 right-[5px] top-[5px]">
          <button
              :disabled="!is_changed_field.attached_peer_box[otherPeerId]"
              class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
              title="Undo Changes"
              @click="undo_connection_changes(otherPeerId);">
            <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
          </button>
        </div>

      </div>
    </div>

  </div>

</template>

<script>
import FastEqual from "fast-deep-equal";
import WireGuardHelper from "@/js/wg-helper.js";


export default {
  name: "connection-islands",
  props: {
    network: {
      type: Object,
      default: {},
    },
    peerId: {
      type: String,
      default: "",
    },
    isNewPeer: {
      type: Boolean,
      default: false,
    },
    api: {
      type: Object,
      default: null,
    }
  },
  data() {
    return {
      attached_static_peer_ids_local: [],
      attached_roaming_peer_ids_local: [],
      connections_local: {
        enabled: {},
        pre_shared_key: {},
        allowed_ips_a_to_b: {},
        allowed_ips_b_to_a: {},
        persistent_keepalive: {},
      },
      island_change_sum: {
        changed_fields: {},
        added_connections: {},
        removed_connections: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      DIV_COLOR_LOOKUP: {
        0: 'bg-green-50',
        1: 'bg-green-100',
        2: 'bg-blue-50',
        '-1': 'bg-red-100',
      },
      is_changed_field: {
        attached_peers_box: 0,
        attached_peer_box: {},
        allowed_ips_a_to_b: {},
        allowed_ips_b_to_a: {},
        persistent_keepalive: {},
      },
    };
  },
  created() {
    this.attached_static_peer_ids_local = this.attached_static_peer_ids;
    this.attached_roaming_peer_ids_local = this.attached_roaming_peer_ids;

    for (const other_peer_id of this.all_attached_peer_ids) {
      const connectionId = WireGuardHelper.getConnectionId(this.peerId, other_peer_id);
      this.connections_local.enabled[other_peer_id] = this.network.connections[connectionId].enabled;
      this.connections_local.pre_shared_key[other_peer_id] = this.network.connections[connectionId].pre_shared_key;
      this.connections_local.allowed_ips_a_to_b[other_peer_id] = this.network.connections[connectionId].allowed_ips_a_to_b;
      this.connections_local.allowed_ips_b_to_a[other_peer_id] = this.network.connections[connectionId].allowed_ips_b_to_a;
      this.connections_local.persistent_keepalive[other_peer_id] = JSON.parse(JSON.stringify(this.network.connections[connectionId].persistent_keepalive));
    }

    if (this.isNewPeer) {
      this.attached_static_peer_ids_local = [this.network.this_peer];
      this.toggleConnection(this.network.this_peer, true);
    }
  },
  methods: {
    check_connection_field_status(other_peer_id, field_name) {
      const connection_id = this._WireGuardHelper_getConnectionId(other_peer_id);
      if (Object.keys(this.network.connections).includes(connection_id) && FastEqual(this.connections_local[field_name][other_peer_id], this.network.connections[connection_id][field_name])) return [0, ''];
      if (!Object.keys(this.connections_local[field_name]).includes(other_peer_id)) return [-1, 'connections_local is out of sync'];
      const ret = WireGuardHelper.checkField(field_name, this.connections_local[field_name][other_peer_id]);
      if (!ret.status) return [-1, ret.msg];
      return [1, ''];
    },
    emit_island_change_sum() {
      this.$emit("updated-change-sum", this.island_change_sum);
    },
    _WireGuardHelper_getConnectionId(otherPeerId) {
      return WireGuardHelper.getConnectionId(this.peerId, otherPeerId);
    },
    async initialize_connection(peer_id) {
      const connection_id = this._WireGuardHelper_getConnectionId(peer_id);
      const default_allowed_ips = this.peerId === this.network.this_peer || peer_id === this.network.this_peer ? '0.0.0.0/0' : this.network.subnet;

      this.connections_local.pre_shared_key[peer_id] = (await this.api.get_wireguard_pre_shared_key()).pre_shared_key;
      this.connections_local.persistent_keepalive[peer_id] = JSON.parse(JSON.stringify(this.network.defaults.connection.persistent_keepalive));
      if (this.network.peers[this.peerId].endpoint.enabled === this.network.peers[peer_id].endpoint.enabled) {
        this.connections_local.allowed_ips_a_to_b[peer_id] = connection_id.startsWith(this.peerId) ? `${this.network.peers[peer_id].address}/32` : `${this.network.peers[this.peerId].address}/32`;
        this.connections_local.allowed_ips_b_to_a[peer_id] = connection_id.startsWith(this.peerId) ? `${this.network.peers[this.peerId].address}/32` : `${this.network.peers[peer_id].address}/32`;
      } else if (this.network.peers[this.peerId].endpoint.enabled &&
          !this.network.peers[peer_id].endpoint.enabled) {
        this.connections_local.allowed_ips_a_to_b[peer_id] = connection_id.startsWith(this.peerId) ? `${this.network.peers[peer_id].address}/32` : default_allowed_ips;
        this.connections_local.allowed_ips_b_to_a[peer_id] = connection_id.startsWith(this.peerId) ? default_allowed_ips : `${this.network.peers[peer_id].address}/32`;
      } else if (!this.network.peers[this.peerId].endpoint.enabled &&
          this.network.peers[peer_id].endpoint.enabled) {
        this.connections_local.allowed_ips_a_to_b[peer_id] = connection_id.startsWith(this.peerId) ? default_allowed_ips : `${this.network.peers[this.peerId].address}/32`;
        this.connections_local.allowed_ips_b_to_a[peer_id] = connection_id.startsWith(this.peerId) ? `${this.network.peers[this.peerId].address}/32` : default_allowed_ips;
      }
    },
    async toggleConnection(peer_id, state = null) {
      this.connections_local.enabled[peer_id] = state ? state : this.connections_local.enabled[peer_id] ? !this.connections_local.enabled[peer_id] : true;

      const connection_id = this._WireGuardHelper_getConnectionId(peer_id);
      if (this.connections_local.enabled[peer_id] && !Object.keys(this.network.connections).includes(connection_id)) {
        await this.initialize_connection(peer_id);
      }

      this.update_added_removed_change_sum();
    },
    update_added_removed_change_sum() {
      const added_connections = {};
      for (const peerId of this.all_attached_peer_ids_local) {
        if (!(this.all_attached_peer_ids.includes(peerId))) {
          added_connections[this._WireGuardHelper_getConnectionId(peerId)] = {
            enabled: this.connections_local.enabled[peerId],
            pre_shared_key: this.connections_local.pre_shared_key[peerId],
            allowed_ips_a_to_b: this.connections_local.allowed_ips_a_to_b[peerId],
            allowed_ips_b_to_a: this.connections_local.allowed_ips_b_to_a[peerId],
            persistent_keepalive: this.connections_local.persistent_keepalive[peerId],
          };
        }
      }
      this.island_change_sum.added_connections = added_connections;

      const removed_connections = {};
      for (const peerId of this.all_attached_peer_ids) {
        if (!(this.all_attached_peer_ids_local.includes(peerId))) {
          removed_connections[this._WireGuardHelper_getConnectionId(peerId)] = {
            enabled: this.connections_local.enabled[peerId],
            pre_shared_key: this.connections_local.pre_shared_key[peerId],
            allowed_ips_a_to_b: this.connections_local.allowed_ips_a_to_b[peerId],
            allowed_ips_b_to_a: this.connections_local.allowed_ips_b_to_a[peerId],
            persistent_keepalive: this.connections_local.persistent_keepalive[peerId],
          };
        }
      }
      this.island_change_sum.removed_connections = removed_connections;
    },
    async undo_connection_changes(otherPeerId) {
      const connection_id = this._WireGuardHelper_getConnectionId(otherPeerId);
      if (!Object.keys(this.network.connections).includes(connection_id)) {
        await this.initialize_connection(otherPeerId);
        return;
      }

      this.connections_local.enabled[otherPeerId] = this.network.connections[connection_id].enabled;
      this.connections_local.pre_shared_key[otherPeerId] = this.network.connections[connection_id].pre_shared_key;
      this.connections_local.persistent_keepalive[otherPeerId] = JSON.parse(JSON.stringify(this.network.connections[connection_id].persistent_keepalive));
      this.connections_local.allowed_ips_a_to_b[otherPeerId] = this.network.connections[connection_id].allowed_ips_a_to_b;
      this.connections_local.allowed_ips_b_to_a[otherPeerId] = this.network.connections[connection_id].allowed_ips_b_to_a;
    },
    async refreshPreSharedKey(otherPeerId) {
      await this.api.get_wireguard_pre_shared_key().then(response => {
        this.connections_local.pre_shared_key[otherPeerId] = response.pre_shared_key;
      });
    }
  },
  emits: ['updated-change-sum'],
  computed: {
    other_static_peer_ids() {
      if (!this.network.peers[this.peerId].endpoint.enabled) {
        return this.network.static_peer_ids;
      }
      const peerId = this.peerId;
      return this.network.static_peer_ids.filter(function (item) {
        return item !== peerId;
      });
    },
    other_roaming_peer_ids() {
      if (this.network.peers[this.peerId].endpoint.enabled) {
        return this.network.roaming_peer_ids;
      }
      const peerId = this.peerId;
      return this.network.roaming_peer_ids.filter(function (item) {
        return item !== peerId;
      });
    },
    attached_static_peer_ids() {
      const ids = [];
      for (const otherPeerId of this.other_static_peer_ids) {
        const connectionId = WireGuardHelper.getConnectionId(otherPeerId, this.peerId);
        if (Object.keys(this.network.connections).includes(connectionId)) ids.push(otherPeerId);
      }
      return ids;
    },
    attached_roaming_peer_ids() {
      const ids = [];
      for (const otherPeerId of this.other_roaming_peer_ids) {
        const connectionId = WireGuardHelper.getConnectionId(otherPeerId, this.peerId);
        if (Object.keys(this.network.connections).includes(connectionId)) ids.push(otherPeerId);
      }
      return ids;
    },
    all_attached_peer_ids() {
      return [...this.attached_static_peer_ids, ...this.attached_roaming_peer_ids];
    },
    selectAllStaticPeers: {
      get() {
        return this.other_static_peer_ids.length ? this.other_static_peer_ids.length === this.attached_static_peer_ids_local.length : false;
      },
      set(value) {
        const attached = [];

        if (value) {
          for (const peerId of this.other_static_peer_ids) {
            attached.push(peerId);
            if (!(peerId in this.attached_static_peer_ids_local)) {
              this.toggleConnection(peerId, true);
            }
          }
        }

        this.attached_static_peer_ids_local = attached;
      },
    },
    selectAllRoamingPeers: {
      get() {
        return this.other_roaming_peer_ids.length ? this.other_roaming_peer_ids.length === this.attached_roaming_peer_ids_local.length : false;
      },
      set(value) {
        const attached = [];

        if (value) {
          for (const peerId of this.other_roaming_peer_ids) {
            attached.push(peerId);
            if (!(peerId in this.attached_roaming_peer_ids_local)) {
              this.toggleConnection(peerId, true);
            }
          }
        }

        this.attached_roaming_peer_ids_local = attached;
      },
    },
    all_attached_peer_ids_local() {
      return [...this.attached_static_peer_ids_local, ...this.attached_roaming_peer_ids_local];
    },
  },
  watch: {
    all_attached_peer_ids_local() {
      this.is_changed_field.attached_peers_box = FastEqual(this.all_attached_peer_ids_local, this.all_attached_peer_ids) ? 0 : 1;
    },
    connections_local: {
      handler() {
        const changed_fields = {};
        for (const other_peer_id of this.all_attached_peer_ids_local) {
          let _msg = "";
          [this.is_changed_field.persistent_keepalive[other_peer_id], _msg] = this.check_connection_field_status(other_peer_id, 'persistent_keepalive');
          [this.is_changed_field.allowed_ips_a_to_b[other_peer_id], _msg] = this.check_connection_field_status(other_peer_id, 'allowed_ips_a_to_b');
          [this.is_changed_field.allowed_ips_b_to_a[other_peer_id], _msg] = this.check_connection_field_status(other_peer_id, 'allowed_ips_b_to_a');

          const connection_id = this._WireGuardHelper_getConnectionId(other_peer_id);
          const connection_details = {
            enabled: this.connections_local.enabled[other_peer_id],
            pre_shared_key: this.connections_local.pre_shared_key[other_peer_id],
            persistent_keepalive: this.connections_local.persistent_keepalive[other_peer_id],
            allowed_ips_a_to_b: this.connections_local.allowed_ips_a_to_b[other_peer_id],
            allowed_ips_b_to_a: this.connections_local.allowed_ips_b_to_a[other_peer_id],
          }

          if (FastEqual(connection_details, this.network.connections[connection_id])) {
            this.is_changed_field.attached_peer_box[other_peer_id] = 0;
          } else {
            if ([this.is_changed_field.persistent_keepalive[other_peer_id], this.is_changed_field.allowed_ips_a_to_b[other_peer_id], this.is_changed_field.allowed_ips_b_to_a[other_peer_id]].includes(-1)) {
              this.is_changed_field.attached_peer_box[other_peer_id] = -1;
            } else {
              this.is_changed_field.attached_peer_box[other_peer_id] = Object.keys(this.network.connections).includes(connection_id) ? 2 : 1;
            }
          }

          const connection_changed_fields = {};
          for (const [fkey, fvalue] of Object.entries(connection_details)) {
            if (!Object.keys(this.network.connections).includes(connection_id)) {
              continue;
            }
            if (!FastEqual(fvalue, this.network.connections[connection_id][fkey])) {
              connection_changed_fields[fkey] = fvalue;
            }
          }
          if (Object.keys(connection_changed_fields).length > 0) {
            changed_fields[connection_id] = connection_changed_fields;
          }
        }
        this.island_change_sum.changed_fields = changed_fields;
        this.update_added_removed_change_sum();
      },
      deep: true,
    },
    island_change_sum: {
      handler() {
        // console.log(JSON.stringify(this.island_change_sum, false, 2));
        this.emit_island_change_sum();
      },
      deep: true
    }
  },
}
</script>

<style scoped>

</style>