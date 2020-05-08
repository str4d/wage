<template>
  <div
    id="app"
    v-bind:class="{
      'one-column-drop-zone': !(encrypting || decrypting),
      'two-column': encrypting,
      'one-column-decrypting': decrypting,
    }"
    @dragover.prevent
    @drop.prevent
  >
    <h1 id="header">rage encrypt all the things!</h1>
    <FileList
      v-if="encrypting"
      v-bind:files="encryptFiles"
      v-on:file-removed="removeFileToEncrypt"
    />
    <DropZone v-if="!decrypting" v-on:files-added="handleFiles" />
    <EncryptPane id="details-pane" v-if="encrypting" />
    <DecryptPane id="details-pane" v-if="decrypting" />
  </div>
</template>

<script>
import DecryptPane from "./components/DecryptPane.vue";
import DropZone from "./components/DropZone.vue";
import EncryptPane from "./components/EncryptPane.vue";
import FileList from "./components/FileList.vue";

export default {
  name: "App",
  components: {
    DecryptPane,
    DropZone,
    EncryptPane,
    FileList
  },
  data() {
    return {
      encryptFiles: [],
      decryptFile: null
    };
  },
  computed: {
    // Are we in "encrypting" mode?
    encrypting() {
      return this.encryptFiles.length;
    },
    // Are we in "decrypting" mode?
    decrypting() {
      return this.decryptFile !== null;
    }
  },
  methods: {
    // Reset application to initial state.
    reset() {
      this.encryptFiles.length = 0;
      this.decryptFile = null;
    },
    // This function is called by the drop zone, so only if we are starting out,
    // or are already encrypting.
    handleFiles(files) {
      if (this.encrypting) {
        // Add more files to encrypt.
        this.addFilesToEncrypt(files);
      } else {
        // Search for a decryptable file.
        var decryptIndex = [...files].findIndex(f => {
          return f.name.endsWith(".age");
        });

        // Decide whether we are encrypting or decrypting.
        if (decryptIndex == -1) {
          this.addFilesToEncrypt(files);
        } else {
          this.startDecrypt(files[decryptIndex]);
        }
      }
    },
    // Encryption methods
    addFilesToEncrypt(files) {
      [...files].forEach(f => {
        this.encryptFiles.push(f);
      });
    },
    removeFileToEncrypt(index) {
      this.encryptFiles.splice(index, 1);
    },
    // Decryption methods
    startDecrypt(file) {
      this.decryptFile = file;
    }
  }
};
</script>

<style>
#app {
  display: grid;
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
  margin-left: 10%;
  margin-right: 10%;
}
.button {
  display: inline-block;
  padding: 10px;
  background: #ccc;
  cursor: pointer;
  border-radius: 5px;
  border: 1px solid #ccc;
}
.button:hover {
  background: #ddd;
}
#header {
  grid-area: header;
}
#file-list {
  grid-area: file-list;
}
#drop-zone {
  grid-area: drop-zone;
}
#details-pane {
  grid-area: details-pane;
}
.one-column-drop-zone {
  grid-template-columns: 10fr;
  grid-template-areas: "header" "drop-zone";
}
.one-column-drop-zone .drop-zone {
  height: 100%;
}
.two-column {
  grid-gap: 20px;
  grid-template-columns: 5fr 5fr;
  grid-template-rows: 1fr 3fr 2fr;
  grid-template-areas: "header header" "file-list details-pane" "drop-zone details-pane";
}
.one-column-decrypting {
  grid-template-columns: 10fr;
  grid-template-areas: "header" "details-pane";
}
</style>
