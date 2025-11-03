<template>

  <div v-if="other_static_peer_ids.length +  other_roaming_peer_ids.length > 0">
    <!-- selection box -->
    <div :class="field_color.attached_peers_box"
         class="my-2 pt-2 pl-1 pr-3 shadow-md border rounded relative">

      <!-- static neighbors -->
      <div v-if="other_static_peer_ids.length > 0" class="mb-2">
        <div class="flex mx-2">
          <span class="text-gray-800 flex items-center">
            <strong class="text-xl mt-[1px]">Attached static peers:</strong>
          </span>
          <checkbox :checked="selectAllStaticPeers" class="ml-auto" label="Select All" size="5"
                    @click="selectAllStaticPeers = !selectAllStaticPeers"></checkbox>
        </div>

        <div class="grid grid-cols-2 px-2 gap-3">
          <div v-for="peerId in other_static_peer_ids" class="overflow-hidden">
            <div class="mt-1 items-center">
              <label class="flex items-center cursor-pointer relative flex-1 min-w-0">
                <input
                    v-model="attached_static_peer_ids_local"
                    :value="peerId"
                    class="h-5 w-5 shrink-0"
                    type="checkbox"
                    @change="toggleConnection(peerId)">

                <span v-if="attached_static_peer_ids_local.includes(peerId)"
                      class="absolute text-white opacity-100 top-13/24 left-0.5 transform -translate-y-1/2 pointer-events-none">
                  <svg class="h-4 w-4" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                    <path clip-rule="evenodd"
                          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                          fill="currentColor" fill-rule="evenodd"></path>
                  </svg>
                </span>

                <span class="align-middle ml-1 truncate">
                  <strong class="text-lg">{{ network.peers[peerId].name }}</strong>
                  {{ network.peers[peerId].address }} ({{ peerId }})
                </span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- roaming neighbors -->
      <div v-if=" other_roaming_peer_ids.length > 0" class="mb-2">

        <div class="flex mx-2">
          <span class="text-gray-800 flex items-center">
            <strong class="text-xl mt-[1px]">Attached roaming peers:</strong>
          </span>
          <checkbox :checked="selectAllRoamingPeers" class="ml-auto" label="Select All" size="5"
                    @click="selectAllRoamingPeers = !selectAllRoamingPeers"></checkbox>
        </div>

        <div class="grid grid-cols-2 px-2 gap-3">
          <div v-for="peerId in other_roaming_peer_ids" class="overflow-hidden">
            <div class="mt-1 items-center">
              <label class="flex items-center cursor-pointer relative flex-1 min-w-0">
                <input
                    v-model="attached_roaming_peer_ids_local"
                    :value="peerId"
                    class="h-5 w-5 shrink-0"
                    type="checkbox"
                    @change="toggleConnection(peerId)">

                <span v-if="attached_roaming_peer_ids_local.includes(peerId)"
                      class="absolute text-white opacity-100 top-13/24 left-0.5 transform -translate-y-1/2 pointer-events-none">
                  <svg class="h-4 w-4" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                    <path clip-rule="evenodd"
                          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                          fill="currentColor" fill-rule="evenodd"></path>
                  </svg>
                </span>

                <span class="align-middle ml-1 truncate">
                  <strong class="text-lg">{{ network.peers[peerId].name }}</strong>
                  {{ network.peers[peerId].address }} ({{ peerId }})
                </span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- Undo Button -->
      <undo-button v-if="field_color.attached_peers_box !== DIV_COLOR_LOOKUP.unchanged"
                   :disabled="field_color.attached_peers_box === DIV_COLOR_LOOKUP.unchanged"
                   alignment-classes="right-[6px] top-[6px]"
                   image-classes="h-7"
                   @click="attached_static_peer_ids_local = attached_static_peer_ids; attached_roaming_peer_ids_local = attached_roaming_peer_ids; update_added_removed_change_sum();">
      </undo-button>
    </div>

    <!-- connection islands -->
    <div v-for="otherPeerId in [...other_static_peer_ids, ...other_roaming_peer_ids]"
         class="relative text-sm">
      <div v-if="all_attached_peer_ids_local.includes(otherPeerId)"
           :class="[field_color.peer_box[otherPeerId]]"
           class="my-2 py-2 pl-1 pr-3 shadow-md border rounded overflow-x-auto whitespace-nowrap highlight-remove-box">

        <!-- enabled checkbox-->
        <div class="ml-2 items-center">
          <label class="flex items-center cursor-pointer relative">
            <input
                v-model="connections_local.enabled[otherPeerId]"
                class="h-5 w-5 shrink-0"
                type="checkbox"
                @change="toggleConnection(peerId)">

            <span v-if="connections_local.enabled[otherPeerId]"
                  class="absolute text-white opacity-100 top-13/24 left-0.5 transform -translate-y-1/2 pointer-events-none">
                  <svg class="h-4 w-4" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                    <path clip-rule="evenodd"
                          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                          fill="currentColor" fill-rule="evenodd"></path>
                  </svg>
                </span>

            <span class="align-middle ml-1">
                  <strong class="text-lg">{{ network.peers[otherPeerId].name }}</strong>
                  {{ network.peers[otherPeerId].address }} ({{ otherPeerId }})
                </span>
          </label>
        </div>

        <!-- connection details  -->
        <hr v-show="connections_local.enabled[otherPeerId]" class="h-1 mt-1 ml-2"/>
        <div v-show="connections_local.enabled[otherPeerId]" class="mt-1 mb-0.5 text-gray-800 text-lg ml-4">

          <!-- Pre Shared Key -->
          <div class="ml-2 flex items-center">
            <field class="inline-block whitespace-pre-wrap" field="PreSharedKey:"></field>
            <refresh-button title="Refresh PreShared Key" @click="refreshPreSharedKey(otherPeerId)"></refresh-button>
            <span class="text-gray-800">{{ connections_local.pre_shared_key[otherPeerId] }}</span>
          </div>

          <!-- Persistent Keepalive -->
          <div class="w-92">
            <input-field v-if="connections_local.persistent_keepalive[otherPeerId]"
                         v-model="connections_local.persistent_keepalive[otherPeerId]"
                         :input-color="field_color.persistent_keepalive[otherPeerId]"
                         value-field="period"
                         :value-prev="stringify_persistent_keepalive(Object.keys(network.connections).includes(_WireGuardHelper_getConnectionId(otherPeerId)) ? network.connections[_WireGuardHelper_getConnectionId(otherPeerId)].persistent_keepalive : network.defaults.connection.persistent_keepalive)"
                         undo-button-alignment-classes="right-[6px] top-[4px]"
                         label="PersistentKeepalive"
                         placeholder="seconds"></input-field>
          </div>

          <!-- Allowed IPs -->
          <div class="text-gray-800 ml-2">
            <div class="mt-0">
              <span class="flex-none">
                <strong>{{ network.peers[peerId].name }}</strong>
                will forward IP subnet(s)
              </span>
              <div class="inline-block relative">
                <input v-if="_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId)"
                       v-model="connections_local.allowed_ips_a_to_b[otherPeerId]"
                       :class="[field_color.allowed_ips_a_to_b[otherPeerId]]"
                       :list="otherPeerId + 'focusPeerName to peerDetails.name'"
                       class="rounded pl-1.5 pt-[2px] pb-[1px] mb-[3px] focus:outline-none focus:ring-0 border-1 border-gray-200 focus:border-gray-400 outline-none w-64 text-lg text-gray-500">
                <input v-else
                       v-model="connections_local.allowed_ips_b_to_a[otherPeerId]"
                       :class="[field_color.allowed_ips_b_to_a[otherPeerId]]"
                       :list="otherPeerId + 'focusPeerName to peerDetails.name'"
                       class="rounded pl-1.5 pt-[2px] pb-[1px] mb-[3px] focus:outline-none focus:ring-0 border-1 border-gray-200 focus:border-gray-400 outline-none w-64 text-lg text-gray-500">
                <!-- Undo Button -->
                <undo-button
                    v-if="_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId) && field_color.allowed_ips_a_to_b[otherPeerId] !== FIELD_COLOR_LOOKUP.unchanged && !isNewPeer"
                    :disabled="field_color.allowed_ips_a_to_b[otherPeerId] === FIELD_COLOR_LOOKUP.unchanged"
                    alignment-classes="right-[5px] top-[2px]"
                    image-classes="h-5"
                    @click="connections_local.allowed_ips_a_to_b[otherPeerId] = stringify_allowed_ips(network.connections[_WireGuardHelper_getConnectionId(otherPeerId)].allowed_ips_a_to_b);">
                </undo-button>
                <undo-button
                    v-else-if="!_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId) && field_color.allowed_ips_b_to_a[otherPeerId] !== FIELD_COLOR_LOOKUP.unchanged && !isNewPeer"
                    :disabled="field_color.allowed_ips_b_to_a[otherPeerId] === FIELD_COLOR_LOOKUP.unchanged"
                    alignment-classes="right-[5px] top-[2px]"
                    image-classes="h-5"
                    @click="connections_local.allowed_ips_b_to_a[otherPeerId] = stringify_allowed_ips(network.connections[_WireGuardHelper_getConnectionId(otherPeerId)].allowed_ips_b_to_a);">
                </undo-button>
              </div>
              <span class="flex-none pr-2"> to <strong>{{ network.peers[otherPeerId].name }}</strong></span>
            </div>
            <div class="mt-0">
              <span class="flex-none">
                <strong>{{ network.peers[otherPeerId].name }}</strong>
                will forward IP subnet(s)
              </span>
              <div class="inline-block relative">
                <input v-if="!_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId)"
                       v-model="connections_local.allowed_ips_a_to_b[otherPeerId]"
                       :class="field_color.allowed_ips_a_to_b[otherPeerId]"
                       :list="otherPeerId + 'peerDetails.name to focusPeerName'"
                       class="rounded pl-1.5 pt-[2px] pb-[1px] focus:outline-none focus:ring-0 border-1 border-gray-200 focus:border-gray-400 outline-none w-64 text-lg text-gray-500 grow">
                <input v-else
                       v-model="connections_local.allowed_ips_b_to_a[otherPeerId]"
                       :class="field_color.allowed_ips_b_to_a[otherPeerId]"
                       :list="otherPeerId + 'peerDetails.name to focusPeerName'"
                       class="rounded pl-1.5 pt-[2px] pb-[1px] focus:outline-none focus:ring-0 border-1 border-gray-200 focus:border-gray-400 outline-none w-64 text-lg text-gray-500 grow">
                <!-- Undo Button -->
                <undo-button
                    v-if="!_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId) && field_color.allowed_ips_a_to_b[otherPeerId] !== FIELD_COLOR_LOOKUP.unchanged && !isNewPeer"
                    :disabled="field_color.allowed_ips_a_to_b[otherPeerId] === FIELD_COLOR_LOOKUP.unchanged"
                    alignment-classes="right-[5px] top-[2px]"
                    image-classes="h-5"
                    @click="connections_local.allowed_ips_a_to_b[otherPeerId] = stringify_allowed_ips(network.connections[_WireGuardHelper_getConnectionId(otherPeerId)].allowed_ips_a_to_b);">
                </undo-button>
                <undo-button
                    v-else-if="_WireGuardHelper_getConnectionId(otherPeerId).startsWith(peerId) && field_color.allowed_ips_b_to_a[otherPeerId] !== FIELD_COLOR_LOOKUP.unchanged && !isNewPeer"
                    :disabled="field_color.allowed_ips_b_to_a[otherPeerId] === FIELD_COLOR_LOOKUP.unchanged"
                    alignment-classes="right-[5px] top-[2px]"
                    image-classes="h-5"
                    @click="connections_local.allowed_ips_b_to_a[otherPeerId] = stringify_allowed_ips(network.connections[_WireGuardHelper_getConnectionId(otherPeerId)].allowed_ips_b_to_a);">
                </undo-button>
              </div>
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

        <!-- Undo Button -->
        <undo-button v-if="field_color.peer_box[otherPeerId] !== DIV_COLOR_LOOKUP.unchanged && !isNewPeer"
                     :disabled="field_color.peer_box[otherPeerId] === DIV_COLOR_LOOKUP.unchanged || isNewPeer"
                     alignment-classes="right-[7px] top-[7px]"
                     image-classes="h-7"
                     @click="undo_connection_changes(otherPeerId);">
        </undo-button>
      </div>
    </div>

  </div>

</template>

<script>
import FastEqual from "fast-deep-equal";
import WireGuardHelper from "@/src/js/wg-helper.js";
import Checkbox from "@/src/components/ui/checkbox.vue";
import Field from "@/src/components/ui/field.vue";
import UndoButton from "@/src/components/ui/buttons/undo.vue";
import InputField from "@/src/components/ui/input-field.vue";
import RefreshButton from "@/src/components/ui/buttons/refresh.vue";
import {validate_conn_allowed_ips_wasm, validate_conn_persistent_keepalive_wasm} from "@/pkg/wg_quickrs_lib.js";


export default {
  name: "connection-islands",
  components: {RefreshButton, InputField, UndoButton, Field, Checkbox},
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
        unchanged: 'bg-white',
        changed: 'enabled:bg-green-200',
        error: 'enabled:bg-red-200',
      },
      DIV_COLOR_LOOKUP: {
        unchanged: 'bg-green-50',
        changed: 'bg-blue-50',
        error: 'bg-red-50',
        new: 'bg-green-100',
      },
      field_color: {
        attached_peers_box: null,
        peer_box: {},
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
      this.connections_local.allowed_ips_a_to_b[other_peer_id] = this.stringify_allowed_ips(this.network.connections[connectionId].allowed_ips_a_to_b);
      this.connections_local.allowed_ips_b_to_a[other_peer_id] = this.stringify_allowed_ips(this.network.connections[connectionId].allowed_ips_b_to_a);
      this.connections_local.persistent_keepalive[other_peer_id] = this.stringify_persistent_keepalive(this.network.connections[connectionId].persistent_keepalive);
    }

    if (this.isNewPeer) {
      this.attached_static_peer_ids_local = [this.network.this_peer];
      this.toggleConnection(this.network.this_peer, true);
    }
  },
  methods: {
    stringify_allowed_ips(allowed_ips) {
      return allowed_ips.join(", ")
    },
    stringify_persistent_keepalive(persistent_keepalive) {
      return {
        enabled: persistent_keepalive.enabled,
        period: `${persistent_keepalive.period}`,
      }
    },
    _WireGuardHelper_getConnectionId(otherPeerId) {
      return WireGuardHelper.getConnectionId(this.peerId, otherPeerId);
    },
    async initialize_connection(peer_id) {
      const connection_id = this._WireGuardHelper_getConnectionId(peer_id);
      const default_allowed_ips = this.peerId === this.network.this_peer || peer_id === this.network.this_peer ? '0.0.0.0/0' : this.network.subnet;

      this.connections_local.pre_shared_key[peer_id] = WireGuardHelper.wg_generate_key();
      this.connections_local.persistent_keepalive[peer_id] = this.stringify_persistent_keepalive(this.network.defaults.connection.persistent_keepalive);
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
            allowed_ips_a_to_b: validate_conn_allowed_ips_wasm(this.connections_local.allowed_ips_a_to_b[peerId]).value,
            allowed_ips_b_to_a: validate_conn_allowed_ips_wasm(this.connections_local.allowed_ips_b_to_a[peerId]).value,
            persistent_keepalive: validate_conn_persistent_keepalive_wasm(this.connections_local.persistent_keepalive[peerId].enabled, this.connections_local.persistent_keepalive[peerId].period).value,
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
            allowed_ips_a_to_b: validate_conn_allowed_ips_wasm(this.connections_local.allowed_ips_a_to_b[peerId]).value,
            allowed_ips_b_to_a: validate_conn_allowed_ips_wasm(this.connections_local.allowed_ips_b_to_a[peerId]).value,
            persistent_keepalive: validate_conn_persistent_keepalive_wasm(this.connections_local.persistent_keepalive[peerId].enabled, this.connections_local.persistent_keepalive[peerId].period).value,
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
      this.connections_local.persistent_keepalive[otherPeerId] = this.stringify_persistent_keepalive(this.network.connections[connection_id].persistent_keepalive);
      this.connections_local.allowed_ips_a_to_b[otherPeerId] = this.stringify_allowed_ips(this.network.connections[connection_id].allowed_ips_a_to_b);
      this.connections_local.allowed_ips_b_to_a[otherPeerId] = this.stringify_allowed_ips(this.network.connections[connection_id].allowed_ips_b_to_a);
    },
    async refreshPreSharedKey(otherPeerId) {
      this.connections_local.pre_shared_key[otherPeerId] = WireGuardHelper.wg_generate_key();
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
      this.field_color.attached_peers_box = FastEqual(this.all_attached_peer_ids_local, this.all_attached_peer_ids) ? this.DIV_COLOR_LOOKUP.unchanged : this.DIV_COLOR_LOOKUP.new;
    },
    connections_local: {
      handler() {
        const changed_fields = {};
        const errors = {};
        for (const other_peer_id of this.all_attached_peer_ids_local) {
          const connection_id = this._WireGuardHelper_getConnectionId(other_peer_id);
          const connection_local_details = {
            enabled: this.connections_local.enabled[other_peer_id],
            pre_shared_key: this.connections_local.pre_shared_key[other_peer_id],
            persistent_keepalive: this.connections_local.persistent_keepalive[other_peer_id],
            allowed_ips_a_to_b: this.connections_local.allowed_ips_a_to_b[other_peer_id],
            allowed_ips_b_to_a: this.connections_local.allowed_ips_b_to_a[other_peer_id],
          };

          let connection_orig_details = {
            enabled: null,
            pre_shared_key: null,
            persistent_keepalive: null,
            allowed_ips_a_to_b: null,
            allowed_ips_b_to_a: null,
          }
          if (this.network.connections[connection_id]) {
            connection_orig_details = this.network.connections[connection_id];
          }

          // Initialize the change sum object
          let connection_change_sum = {
            errors: {},
            changed_fields: {}
          };

          if (connection_local_details.enabled !== connection_orig_details.enabled) {
            connection_change_sum.changed_fields.enabled = connection_local_details.enabled;
          }

          if (connection_local_details.pre_shared_key !== connection_orig_details.pre_shared_key) {
            connection_change_sum.changed_fields.pre_shared_key = connection_local_details.pre_shared_key;
          }

          // persistent_keepalive
          [this.field_color.persistent_keepalive[other_peer_id], connection_change_sum] = WireGuardHelper.validateField(
              'persistent_keepalive',
              validate_conn_persistent_keepalive_wasm,
              connection_orig_details.persistent_keepalive,
              connection_change_sum,
              this.FIELD_COLOR_LOOKUP,
              connection_local_details.persistent_keepalive.enabled,  // validator arg
              connection_local_details.persistent_keepalive.period    // validator arg
          );

          // allowed_ips_a_to_b
          [this.field_color.allowed_ips_a_to_b[other_peer_id], connection_change_sum] = WireGuardHelper.validateField(
              'allowed_ips_a_to_b',
              validate_conn_allowed_ips_wasm,
              connection_orig_details.allowed_ips_a_to_b,
              connection_change_sum,
              this.FIELD_COLOR_LOOKUP,
              connection_local_details.allowed_ips_a_to_b,  // validator arg
          );

          // allowed_ips_b_to_a
          [this.field_color.allowed_ips_b_to_a[other_peer_id], connection_change_sum] = WireGuardHelper.validateField(
              'allowed_ips_b_to_a',
              validate_conn_allowed_ips_wasm,
              connection_orig_details.allowed_ips_b_to_a,
              connection_change_sum,
              this.FIELD_COLOR_LOOKUP,
              connection_local_details.allowed_ips_b_to_a,  // validator arg
          );

          connection_change_sum.errors = Object.fromEntries(
              Object.entries(connection_change_sum.errors).filter(([key, obj]) => obj !== null)
          );
          if (Object.keys(connection_change_sum.errors).length > 0) {
            errors[connection_id] = connection_change_sum.errors;
            this.field_color.peer_box[other_peer_id] = this.DIV_COLOR_LOOKUP.error;
            continue;
          }
          connection_change_sum.changed_fields = Object.fromEntries(
              Object.entries(connection_change_sum.changed_fields).filter(([key, obj]) => obj !== null)
          );
          if (Object.keys(connection_change_sum.changed_fields).length > 0) {
            changed_fields[connection_id] = connection_change_sum.changed_fields;
            this.field_color.peer_box[other_peer_id] = this.network.connections[connection_id] ? this.DIV_COLOR_LOOKUP.changed : this.DIV_COLOR_LOOKUP.new;
            continue;
          }
          this.field_color.peer_box[other_peer_id] = this.DIV_COLOR_LOOKUP.unchanged;
        }
        this.island_change_sum.errors = errors;
        this.island_change_sum.changed_fields = changed_fields;
        this.update_added_removed_change_sum();
      },
      deep: true,
    },
    island_change_sum: {
      handler() {
        console.log(JSON.stringify(this.island_change_sum));
        this.$emit("updated-change-sum", this.island_change_sum)
      },
      deep: true
    }
  },
}
</script>

<style scoped>

</style>