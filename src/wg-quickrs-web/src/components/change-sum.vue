<template>

  <div class="text-sm text-gray-500 whitespace-pre grid grid-cols-2 gap-1">
    <div v-if="is_full(pruned_errors)" class="col-span-2 bg-red-100 rounded-md overflow-scroll">
      <div class="flex items-center justify-center bg-red-200 rounded-md p-1">
        <strong class="text-gray-600">Errors</strong>
      </div>
      <div class="text-sm text-gray-700 p-1">
        {{ JSON.stringify(pruned_errors, false, 2) }}
      </div>
    </div>
    <div v-if="is_full(pruned_changed_fields)" class="col-span-2 bg-blue-100 rounded-md overflow-scroll">
      <div class="flex items-center justify-center bg-blue-200 rounded-md p-1">
        <strong class="text-gray-600">Changed Fields</strong>
      </div>
      <div class="text-sm text-gray-700 p-1">
        {{ JSON.stringify(pruned_changed_fields, false, 2) }}
      </div>
    </div>
    <div v-if="is_full(changeSum.added_peers)" class="col-span-2 bg-green-100 rounded-md overflow-scroll">
      <div class="flex items-center justify-center bg-green-200 rounded-md p-1">
        <strong class="text-gray-600">Added Peer</strong>
      </div>
      <div class="text-sm text-gray-700 p-1">
        {{ JSON.stringify(padded_added_peers, false, 2) }}
      </div>
    </div>
    <div v-if="is_full(changeSum.added_connections)" class="col-span-2 bg-green-100 rounded-md overflow-scroll">
      <div class="flex items-center justify-center bg-green-200 rounded-md p-1">
        <strong class="text-gray-600">Added Connections</strong>
      </div>
      <div class="text-sm text-gray-700 p-1">
        {{ JSON.stringify(changeSum.added_connections, false, 2) }}
      </div>
    </div>
    <div v-if="is_full(changeSum.removed_connections)" class="col-span-2 bg-red-100 rounded-md overflow-scroll">
      <div class="flex items-center justify-center bg-red-200 rounded-md p-1">
        <strong class="text-gray-600">Removed Connections</strong>
      </div>
      <div class="text-sm text-gray-700 p-1">
        {{ JSON.stringify(changeSum.removed_connections, false, 2) }}
      </div>
    </div>
    <div class="bg-gray-100 rounded-md overflow-scroll">
      <div class="flex items-center justify-center bg-gray-200 rounded-md p-1">
        <strong class="text-gray-600">Old Configuration</strong>
      </div>
      <span class="text-sm text-gray-700 p-1">
        {{ JSON.stringify(pruned_network, false, 2) }}
      </span>
    </div>
    <div class="bg-green-100 rounded-md overflow-scroll">
      <div class="flex items-center justify-center bg-green-200 rounded-md p-1">
        <strong class="text-gray-600">New Configuration</strong>
      </div>
      <span class="text-sm text-gray-700 p-1">
        {{ JSON.stringify(new_network, false, 2) }}
      </span>
    </div>
  </div>

</template>

<script>


export default {
  name: "change-sum",
  props: {
    changeSum: {
      type: Object,
      default: {
        errors: {
          peers: {},
          connections: {},
        },
        changed_fields: {
          peers: {},
          connections: {},
        },
        added_connections: {},
        removed_connections: {}
      },
    },
    peerId: {
      type: String,
      default: "",
    },
    network: {
      type: Object,
      default: {},
    },
    dialogId: {
      type: String,
      default: "",
    }
  },
  methods: {
    is_full(field) {
      if (field === undefined) {
        return false;
      }
      return Object.keys(field).length !== 0;

    }
  },
  computed: {
    pruned_errors() {
      const pruned_errors = {};
      if (this.changeSum.errors.peers) {
        if (this.changeSum.errors.peers[this.peerId]) {
          if (Object.keys(this.changeSum.errors.peers[this.peerId]).length) {
            pruned_errors.peers = this.changeSum.errors.peers;
          }
        }
      }
      if (this.changeSum.errors.connections) {
        if (Object.keys(this.changeSum.errors.connections).length) {
          pruned_errors.connections = this.changeSum.errors.connections;
        }
      }
      if (this.changeSum.errors.amnezia_parameters) {
        if (Object.keys(this.changeSum.errors.amnezia_parameters).length) {
          pruned_errors.amnezia_parameters = this.changeSum.errors.amnezia_parameters;
        }
      }
      if (this.changeSum.errors.defaults) {
        if (Object.keys(this.changeSum.errors.defaults).length) {
          pruned_errors.defaults = this.changeSum.errors.defaults;
        }
      }
      return pruned_errors;
    },
    pruned_changed_fields() {
      if (!Object.keys(this.changeSum).includes("changed_fields")) {
        return {};
      }

      const pruned_changed_fields = {};
      if (this.changeSum.changed_fields.peers) {
        if (Object.keys(this.changeSum.changed_fields.peers[this.peerId]).length) {
          pruned_changed_fields.peers = this.changeSum.changed_fields.peers;
        }
      }
      if (this.changeSum.changed_fields.connections) {
        if (Object.keys(this.changeSum.changed_fields.connections).length) {
          pruned_changed_fields.connections = this.changeSum.changed_fields.connections;
        }
      }
      if (this.changeSum.changed_fields.amnezia_parameters) {
        if (Object.keys(this.changeSum.changed_fields.amnezia_parameters).length) {
          pruned_changed_fields.amnezia_parameters = this.changeSum.changed_fields.amnezia_parameters;
        }
      }
      if (this.changeSum.changed_fields.defaults) {
        if (Object.keys(this.changeSum.changed_fields.defaults).length) {
          pruned_changed_fields.defaults = this.changeSum.changed_fields.defaults;
        }
      }
      return pruned_changed_fields;
    },
    padded_added_peers() {
      return {peers: this.changeSum.added_peers};
    },
    pruned_network() {
      let pruned_network = {};
      if (this.peerId) {
        pruned_network = { peers: {}, connections: {} };
        pruned_network.peers[this.peerId] = this.network.peers[this.peerId];

        for (let connection_id in this.network.connections) {
          if (connection_id.includes(this.peerId)) {
            pruned_network.connections[connection_id] = this.network.connections[connection_id];
          }
        }
      } else if (this.dialogId === "network-defaults") {
        pruned_network.defaults = this.network.defaults;
      } else if (this.dialogId === "amnezia-settings") {
        pruned_network = { defaults: { peer: {} } };
        pruned_network.defaults.peer.amnezia_parameters = this.network.defaults.peer.amnezia_parameters;
        pruned_network.amnezia_parameters = this.network.amnezia_parameters;
      }
      return pruned_network;
    },
    new_network() {
      let new_network = JSON.parse(JSON.stringify(this.pruned_network));

      if (this.changeSum.changed_fields) {
        // peer changes
        if (this.changeSum.changed_fields.peers) {
          for (const peer_id in this.changeSum.changed_fields.peers) {
            for (const peer_field in this.changeSum.changed_fields.peers[peer_id]) {
              if (peer_field === "scripts") {
                for (const script_field in this.changeSum.changed_fields.peers[peer_id].scripts) {
                  new_network.peers[peer_id].scripts[script_field] = this.changeSum.changed_fields.peers[peer_id].scripts[script_field];
                }
                continue;
              }
              new_network.peers[peer_id][peer_field] = this.changeSum.changed_fields.peers[peer_id][peer_field];
            }
          }
        }

        // connection changes
        if (this.changeSum.changed_fields.connections) {
          for (const connection_id in this.changeSum.changed_fields.connections) {
            for (const connection_field in this.changeSum.changed_fields.connections[connection_id]) {
              new_network.connections[connection_id][connection_field] = this.changeSum.changed_fields.connections[connection_id][connection_field];
            }
          }
        }

        // amnezia_parameters changes
        if (this.changeSum.changed_fields.amnezia_parameters) {
          for (const amnezia_field in this.changeSum.changed_fields.amnezia_parameters) {
            new_network.amnezia_parameters[amnezia_field] = this.changeSum.changed_fields.amnezia_parameters[amnezia_field];
          }
        }

        // defaults changes
        if (this.changeSum.changed_fields.defaults) {
          for (const defaults_field in this.changeSum.changed_fields.defaults.peer) {
            if (defaults_field === "amnezia_parameters") {
              for (const amnezia_field in this.changeSum.changed_fields.defaults.peer.amnezia_parameters) {
                new_network.defaults.peer.amnezia_parameters[amnezia_field] = this.changeSum.changed_fields.defaults.peer.amnezia_parameters[amnezia_field];
              }
              continue;
            }
            new_network.defaults.peer[defaults_field] = this.changeSum.changed_fields.defaults.peer[defaults_field];
          }
          for (const defaults_field in this.changeSum.changed_fields.defaults.connection) {
            new_network.defaults.connection[defaults_field] = this.changeSum.changed_fields.defaults.connection[defaults_field];
          }
        }
      }

      if (this.changeSum.added_connections) {
        for (const added_connection_id in this.changeSum.added_connections) {
          new_network.connections[added_connection_id] = this.changeSum.added_connections[added_connection_id];
        }
      }

      if (this.changeSum.removed_connections) {
        for (const removed_connection_id in this.changeSum.removed_connections) {
          delete new_network.connections[removed_connection_id];
        }
      }

      return new_network;
    },
  }
}
</script>

<style scoped>

</style>