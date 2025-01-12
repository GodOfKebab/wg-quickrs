<template>

  <div v-if="staticPeerIdsExcludingSelf.length + roamingPeerIdsExcludingSelf.length > 0">
    <div
        :class="[color.selectionDiv, _FastEqual(attachedStaticPeerIdsLocal, attachedStaticPeerIds) && _FastEqual(attachedRoamingPeerIdsLocal, attachedRoamingPeerIds) ? '' : 'highlight-undo-box']"
        class="my-2 p-1 shadow-md border rounded relative">
      <div v-if="staticPeerIdsExcludingSelf.length > 0">
        <div class="text-gray-800">
          Attached static peers:
        </div>
        <div class="form-check mt-1">
          <label class="form-check-label inline-block text-gray-800 cursor-pointer text-sm">
            <input
                v-model="selectAllStaticPeers"
                class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 mt-1 align-top bg-no-repeat bg-center bg-contain float-left mr-2 cursor-pointer"
                type="checkbox">
            <span>Select All</span>
          </label>
        </div>
        <div class="grid grid-cols-2">
          <div v-for="peerId in staticPeerIdsExcludingSelf"
               class="relative overflow-hidden">
            <div class="form-check truncate">
              <label>
                <input
                    v-model="attachedStaticPeerIdsLocal"
                    :value="peerId"
                    class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 mt-1 align-top bg-no-repeat bg-center bg-contain float-left mr-2 cursor-pointer"
                    type="checkbox"
                    @change="isConnectionEnabled[peerId] = true;">
                <span class="text-gray-800 cursor-pointer text-xs">
                           <strong class="text-sm">{{
                               this.network.peers[peerId].name
                             }}</strong> {{ this.network.peers[peerId].address }} ({{
                    peerId
                  }})
                         </span>
              </label>
            </div>
          </div>
        </div>
      </div>
      <div v-if="roamingPeerIdsExcludingSelf.length > 0">
        <div class="text-gray-800">
          Attached roaming peers:
        </div>
        <div class="form-check mt-1">
          <label class="form-check-label inline-block text-gray-800 cursor-pointer text-sm">
            <input
                v-model="selectAllRoamingPeers"
                class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 mt-1 align-top bg-no-repeat bg-center bg-contain float-left mr-2 cursor-pointer inline-block"
                type="checkbox">
            <span>Select All</span>
          </label>
        </div>
        <div class="grid grid-cols-2">
          <div v-for="peerId in roamingPeerIdsExcludingSelf"
               class="relative overflow-hidden">
            <div class="form-check truncate">
              <label>
                <input
                    v-model="attachedRoamingPeerIdsLocal"
                    :value="peerId"
                    class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 mt-1 align-top bg-no-repeat bg-center bg-contain float-left mr-2 cursor-pointer"
                    type="checkbox"
                    @change="isConnectionEnabled[peerId] = true;">
                <span class="text-gray-800 cursor-pointer text-xs">
                           <strong class="text-sm">{{
                               this.network.peers[peerId].name
                             }}</strong> {{ this.network.peers[peerId].address }} ({{
                    peerId
                  }})
                         </span>
              </label>
            </div>
          </div>
        </div>
      </div>
      <div
          v-if="!(_FastEqual(attachedStaticPeerIdsLocal, attachedStaticPeerIds) && _FastEqual(attachedRoamingPeerIdsLocal, attachedRoamingPeerIds))"
          class="inline-block float-right absolute z-20 right-[0.2rem] top-[0rem]">
        <button
            :disabled="_FastEqual(attachedStaticPeerIdsLocal, attachedStaticPeerIds) && _FastEqual(attachedRoamingPeerIdsLocal, attachedRoamingPeerIds)"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white opacity-0 transition undo-button-itself"
            title="Undo Changes"
            @click="attachedStaticPeerIdsLocal = attachedStaticPeerIds; attachedRoamingPeerIdsLocal = attachedRoamingPeerIds">
          <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>


    <div v-for="otherPeerId in Object.assign([], staticPeerIdsExcludingSelf, roamingPeerIdsExcludingSelf)"
         class="relative">
      <div v-if="attachedStaticPeerIdsLocal.includes(otherPeerId) || attachedRoamingPeerIdsLocal.includes(otherPeerId)"
           :class="[color.attachedPeerDiv[otherPeerId], connectionChanged[otherPeerId] ? 'highlight-undo-box' : '']"
           class="my-2 p-1 shadow-md border rounded bg-blue-50 overflow-x-auto whitespace-nowrap highlight-remove-box">

        <div class=" ml-1">
          <div class="form-check">
            <label class="form-check-label inline-block text-gray-800 cursor-pointer text-sm">
              <input
                  v-model="this.isConnectionEnabled[otherPeerId]"
                  class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 mt-1 align-top bg-no-repeat bg-center bg-contain float-left mr-2 cursor-pointer"
                  type="checkbox"
                  @change="colorRefresh += 1">
              <span class="text-gray-800 text-xs">
                           <strong class="text-sm">{{ network.peers[otherPeerId].name }}</strong>
                           {{ network.peers[otherPeerId].address }}
                           ({{ otherPeerId }})
                         </span>
            </label>
          </div>
        </div>


      </div>
    </div>
  </div>

</template>

<script>
import FastEqual from "fast-deep-equal";
import WireGuardHelper from "../../js/wg-helper.js";


export default {
  name: "connection-islands",
  props: {
    network: {
      type: Object,
      default: {},
    },
    peerId: {
      type: Object,
      default: {},
    },
    value: {
      type: Object,
      default: {
        context: 'edit',
        addedFields: {},
        removedFields: {},
        changedFields: {},
        error: null,
      },
    },
  },
  data() {
    return {
      connections_local: {},
      colorRefresh: 0,
      connectionChanged: {},
      attachedStaticPeerIdsLocal: [],
      attachedRoamingPeerIdsLocal: [],
      isConnectionEnabled: {},
      allowed_ips_a_to_b: {},
      allowed_ips_b_to_a: {},
      persistent_keepalive_enabled: {},
      persistent_keepalive_value: {},
    };
  },
  created() {
    this.connections_local = this.value;

    // To enforce order of static > roaming connections when listed in the view
    for (const peerId of this.staticPeerIdsExcludingSelf) {
      const connectionId = WireGuardHelper.getConnectionId(peerId, this.peerId);
      console.log(`static connectionId ${connectionId}`)
      if (Object.keys(this.network.connections).includes(connectionId)) this.attachedStaticPeerIdsLocal.push(peerId);
    }
    for (const peerId of this.roamingPeerIdsExcludingSelf) {
      const connectionId = WireGuardHelper.getConnectionId(peerId, this.peerId);
      console.log(`roaming connectionId ${connectionId}`)
      if (Object.keys(this.network.connections).includes(connectionId)) this.attachedRoamingPeerIdsLocal.push(peerId);
    }
    console.log(`this.network.connections ${JSON.stringify(this.network.connections)}`)
    // console.log(`... ${this.attachedStaticPeerIdsLocal} ${this.attachedRoamingPeerIdsLocal}`)

    // for (const peerId of [...Object.keys(this.value.staticPeers), ...Object.keys(this.value.roamingPeers)]) {
    //   console.log(peerId)
    //   const connectionId = WireGuardHelper.getConnectionId(this.peerId, peerId);
    //   this.connectionChanged[peerId] = false;
    // }

    this.value.addedFields = {};
    this.value.removedFields = {};
    this.value.changedFields = {};
    this.value.error = null;
  },
  methods: {
    _FastEqual(a, b) {
      return FastEqual(a, b);
    },
    _WireGuardHelper_getConnectionId(otherPeerId) {
      return WireGuardHelper.getConnectionId(this.peerId, otherPeerId);
    },
    // async refreshConnectionEditKeys(connectionId) {
    //   const {preSharedKey} = await this.getNewPreSharedKey();
    //   this.value.preSharedKey[peerId] = preSharedKey;
    //   this.colorRefresh += 1;
    // },
  },
  emits: ['update:value'],
  computed: {
    staticPeerIdsExcludingSelf() {
      const ids = [];
      for (const peerId of this.network.staticPeerIds) {
        if (peerId === this.peerId) continue;
        ids.push(peerId);
      }
      return ids;
    },
    roamingPeerIdsExcludingSelf() {
      const ids = [];
      for (const peerId of this.network.roamingPeerIds) {
        if (peerId === this.peerId) continue;
        ids.push(peerId);
      }
      return ids;
    },
    attachedStaticPeerIds() {
      const ids = [];
      for (const staticPeerId of this.staticPeerIdsExcludingSelf) {
        const connectionId = WireGuardHelper.getConnectionId(staticPeerId, this.peerId);
        if (Object.keys(this.network.connections).includes(connectionId)) ids.push(staticPeerId);
      }
      return ids;
    },
    attachedRoamingPeerIds() {
      const ids = [];
      for (const peerId of this.roamingPeerIdsExcludingSelf) {
        const connectionId = WireGuardHelper.getConnectionId(peerId, this.peerId);
        if (Object.keys(this.network.connections).includes(connectionId)) ids.push(peerId);
      }
      return ids;
    },
    selectAllStaticPeers: {
      get() {
        return this.staticPeerIdsExcludingSelf.length ? this.staticPeerIdsExcludingSelf.length === this.attachedStaticPeerIdsLocal.length : false;
      },
      set(value) {
        const attached = [];

        if (value) {
          for (const peerId of this.staticPeerIdsExcludingSelf) {
            attached.push(peerId);
            if (!(peerId in this.attachedStaticPeerIdsLocal)) {
              this.isConnectionEnabled[peerId] = true;
            }
          }
        }

        this.attachedStaticPeerIdsLocal = attached;
      },
    },
    selectAllRoamingPeers: {
      get() {
        return this.roamingPeerIdsExcludingSelf.length ? this.roamingPeerIdsExcludingSelf.length === this.attachedRoamingPeerIdsLocal.length : false;
      },
      set(value) {
        const attached = [];

        if (value) {
          for (const peerId of this.roamingPeerIdsExcludingSelf) {
            attached.push(peerId);
            if (!(peerId in this.attachedRoamingPeerIdsLocal)) {
              this.isConnectionEnabled[peerId] = true;
            }
          }
        }

        this.attachedRoamingPeerIdsLocal = attached;
      },
    },
    allAttachedPeers() {
      return [...this.attachedStaticPeerIds, ...this.attachedRoamingPeerIds];
    },
    allAttachedPeersLocal() {
      return [...this.attachedStaticPeerIdsLocal, ...this.attachedRoamingPeerIdsLocal];
    },
    color() {
      this.colorRefresh &&= this.colorRefresh;
      const color = {
        allowed_ips_a_to_b: {},
        allowed_ips_b_to_a: {},
        persistent_keepalive: {},
        attachedPeerDiv: {},
        selectionDiv: WireGuardHelper.checkField('peerCount', this.allAttachedPeersLocal) ? 'bg-green-50' : 'bg-red-50',
      };
      const addedFields = {};
      const changedFields = {};
      let error = null;
      for (const peerId of this.allAttachedPeers) {
        const connectionId = WireGuardHelper.getConnectionId(this.peerId, peerId);
        try {
          addedFields[peerId] = {};
          changedFields[peerId] = {};
          // eslint-disable-next-line no-nested-ternary
          color.allowed_ips_a_to_b[peerId] = this.value.context === 'create' || !this.allAttachedPeers.includes(peerId) || this.allowed_ips_a_to_b[peerId] !== this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].allowed_ips_a_to_b
              ? WireGuardHelper.checkField('allowedIPs', this.allowed_ips_a_to_b[peerId]) ? 'bg-green-200' : 'bg-red-200' : 'bg-white';
          if (this.allowed_ips_a_to_b[peerId] !== this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].allowed_ips_a_to_b) {
            changedFields[peerId].allowed_ips_a_to_b = this.allowed_ips_a_to_b[peerId];
          }
          error = color.allowed_ips_a_to_b[peerId] === 'bg-red-200' ? `${connectionId}'s 'allowed_ips_a_to_b' field` : error;

          // eslint-disable-next-line no-nested-ternary
          color.allowed_ips_b_to_a[peerId] = this.value.context === 'create' || !this.allAttachedPeers.includes(peerId) || this.value.allowed_ips_b_to_a[peerId] !== this.network.peers[peerId]
              ? WireGuardHelper.checkField('allowedIPs', this.allowed_ips_b_to_a[peerId]) ? 'bg-green-200' : 'bg-red-200' : 'bg-white';
          if (this.value.allowed_ips_b_to_a[peerId] !== this.network.peers[peerId]) {
            changedFields[peerId].allowed_ips_b_to_a = this.allowed_ips_b_to_a[peerId];
          }
          error = color.allowed_ips_b_to_a[peerId] === 'bg-red-200' ? `${connectionId}'s 'allowed_ips_b_to_a' field` : error;

          const changedpersistent_keepalive = {};
          // eslint-disable-next-line no-nested-ternary
          color.persistent_keepalive[peerId] = !this.allAttachedPeers.includes(peerId) || this.value.persistent_keepaliveEnabled[peerId] !== this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].persistent_keepalive.enabled
          || this.value.persistent_keepaliveValue[peerId] !== this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].persistent_keepalive.value
              ? WireGuardHelper.checkField('persistent_keepalive', this.persistent_keepalive_value[peerId]) ? 'bg-green-200' : 'bg-red-200' : 'bg-white';
          if (this.value.persistent_keepaliveEnabled[peerId] !== this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].persistent_keepalive.enabled) {
            changedpersistent_keepalive.enabled = this.value.persistent_keepaliveEnabled[connectionId];
          }
          if (this.value.persistent_keepaliveValue[peerId] !== this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].persistent_keepalive.value) {
            changedpersistent_keepalive.value = this.persistent_keepalive_value[peerId];
          }
          if (Object.keys(changedpersistent_keepalive).length > 0) changedFields[peerId].persistent_keepalive = changedpersistent_keepalive;
          error = color.persistent_keepalive[peerId] === 'bg-red-200' ? `${connectionId}'s 'persistent_keepalive' field` : error;

          if (this.value.preSharedKey[peerId] !== this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].pre_shared_key) {
            changedFields[peerId].preSharedKey = this.value.preSharedKey[connectionId];
          }

          this.connectionChanged[peerId] = Object.keys(changedFields[peerId]).length !== 0;
          if (Object.keys(changedFields[peerId]).length === 0) delete changedFields[peerId];

          if (!this.allAttachedPeers.includes(peerId)) {
            addedFields[peerId] = {
              enabled: this.isConnectionEnabled[peerId],
              allowed_ips_a_to_b: this.allowed_ips_a_to_b[peerId],
              allowed_ips_b_to_a: this.allowed_ips_b_to_a[peerId],
              persistent_keepalive: {
                enabled: this.value.persistent_keepaliveEnabled[connectionId],
                value: this.persistent_keepalive_value[peerId],
              },
            };
          } else {
            delete addedFields[peerId];
          }

          // eslint-disable-next-line no-nested-ternary
          color.attachedPeerDiv[peerId] = ![color.allowed_ips_a_to_b[peerId], color.allowed_ips_b_to_a[peerId], this.value.persistent_keepaliveEnabled[peerId] ? color.persistent_keepalive[peerId] : ''].includes('bg-red-200') ? this.isConnectionEnabled[peerId] ? this.connectionChanged[peerId] || (!this.allAttachedPeers.includes(peerId) && this.value.context === 'edit') ? 'bg-green-100' : 'bg-green-50' : 'bg-red-50' : 'bg-red-100';
        } catch (e) {
          this.connectionChanged[peerId] = true;
          for (const colorField of Object.keys(color)) {
            if (colorField === 'selectionDiv') continue;
            color[colorField][peerId] = 'bg-red-50';
          }
          console.log(e);
        }
      }

      const removedFields = {};
      for (const peerId of this.allAttachedPeers) {
        if (!this.allAttachedPeers.includes(peerId)) {
          const connectionId = WireGuardHelper.getConnectionId(peerId, this.peerId);
          removedFields[peerId] = {
            enabled: this.rollbackData.isConnectionEnabled[peerId],
            allowed_ips_a_to_b: this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].allowed_ips_a_to_b,
            allowed_ips_b_to_a: this.network.peers[peerId],
            persistent_keepalive: {
              enabled: this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].persistent_keepalive.enabled,
              value: this.network.connections[WireGuardHelper.getConnectionId(this.peerId, peerId)].persistent_keepalive.value,
            },
          };
        }
      }

      this.value.addedFields = addedFields;
      this.value.removedFields = removedFields;
      this.value.changedFields = changedFields;
      this.value.error = error;

      return color;
    },
  },
}
</script>

<style scoped>

</style>