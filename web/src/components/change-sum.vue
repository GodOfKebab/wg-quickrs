<template>

  <div class="text-sm text-gray-500 whitespace-pre grid grid-cols-2 gap-1">
    <div v-show="changeSum.errors" class="col-span-2 bg-red-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-red-200 p-1">Errors</strong>
      {{ JSON.stringify(changeSum.errors, false, 2) }}
    </div>
    <div v-show="changeSum.changed_fields" class="col-span-2 bg-blue-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-blue-200 p-1">Changed Fields</strong>
      {{ JSON.stringify(changeSum.changed_fields, false, 2) }}
    </div>
    <div v-show="changeSum.added_connections" class="col-span-2 bg-green-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-green-200 p-1">Added Connections</strong>
      {{ JSON.stringify(changeSum.added_connections, false, 2) }}
    </div>
    <div v-show="changeSum.removed_connections" class="col-span-2 bg-red-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-red-200 p-1">Removed Connections</strong>
      {{ JSON.stringify(changeSum.removed_connections, false, 2) }}
    </div>
    <div class="bg-gray-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-gray-200 p-1">Old Configuration</strong>
      <div class="p-1">{{
          JSON.stringify({peers: pruned_network.peers, connections: pruned_network.connections}, false, 2)
        }}
      </div>
    </div>
    <div class="bg-green-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-green-200 p-1">New Configuration</strong>
      <div class="p-1">{{
          JSON.stringify({peers: new_network.peers, connections: new_network.connections}, false, 2)
        }}
      </div>
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
  },
  computed: {
    pruned_network() {
      let pruned_network = {peers: {}, connections: {}};
      pruned_network.peers[this.peerId] = this.network.peers[this.peerId];

      for (let connection_id in this.network.connections) {
        if (connection_id.includes(this.peerId)) {
          pruned_network.connections[connection_id] = this.network.connections[connection_id];
        }
      }
      return pruned_network;
    },
    new_network() {
      let new_network = JSON.parse(JSON.stringify(this.pruned_network));

      if (this.changeSum.changed_fields) {
        // peer changes
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

        // connection changes
        for (const connection_id in this.changeSum.changed_fields.connections) {
          for (const connection_field in this.changeSum.changed_fields.connections[connection_id]) {
            new_network.connections[connection_id][connection_field] = this.changeSum.changed_fields.connections[connection_id][connection_field];
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