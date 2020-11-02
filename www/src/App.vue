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
    <p v-if="errorMsg"><b>Error: {{ errorMsg }}</b></p>
    <FileList
      v-if="encrypting"
      v-bind:files="encryptFiles"
      v-on:file-removed="removeFileToEncrypt"
    />
    <DropZone v-if="!decrypting" v-on:files-added="handleFiles" />
    <EncryptPane id="details-pane" v-if="encrypting" />
    <DecryptPane
      id="details-pane"
      v-if="decrypting"
      v-bind:fileDecrypted="fileDecrypted"
      v-bind:needPassphrase="needPassphrase"
      v-on:decrypt-with-passphrase="decryptWithPassphrase"
      v-on:download-file="downloadDecryptedFile"
    />
    <div id="footer">
      <p>
        This is an
        <strong>EXPERIMENTAL</strong> alpha version; <strong>DO NOT</strong> use
        it for real files yet.
      </p>
      <p>
        <a href="https://str4d.xyz/wage">Source available here!</a> Powered by
        <a href="https://str4d.xyz/rage">rage</a>.
      </p>
    </div>
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
    FileList,
  },
  data() {
    return {
      wasm: null,
      errorMsg: null,
      encryptFiles: [],
      decryptFile: null,
      decryptor: null,
      decryptedStream: null,
      downloadStream: null,
    };
  },
  beforeCreate() {
    // WASM needs to be imported asynchronously.
    import("wage").then((wasm) => {
      this.wasm = wasm;
    });
  },
  created() {
    window.addEventListener("beforeunload", this.checkDownloads);
    window.addEventListener("unload", this.cancelDownloads);
  },
  computed: {
    // Are we in "encrypting" mode?
    encrypting() {
      return this.encryptFiles.length;
    },
    // Are we in "decrypting" mode?
    decrypting() {
      return this.decryptFile !== null;
    },
    // Do we need a passphrase from the user?
    needPassphrase() {
      return this.decryptor !== null && this.decryptor.requires_passphrase();
    },
    // Have we successfully decrypted the file?
    fileDecrypted() {
      return this.decryptedStream !== null;
    },
    fileDownloading() {
      return this.downloadStream !== null;
    },
  },
  methods: {
    // Reset application to initial state.
    reset() {
      this.errorMsg = null;
      this.encryptFiles = [];
      this.decryptFile = null;
      this.decryptor = null;
      this.decryptedStream = null;
      this.downloadStream = null;
    },
    // This function is called by the drop zone, so only if we are starting out,
    // or are already encrypting.
    handleFiles(files) {
      this.errorMsg = null;

      if (this.encrypting) {
        // Add more files to encrypt.
        this.addFilesToEncrypt(files);
      } else {
        // Search for a decryptable file.
        var decryptIndex = [...files].findIndex((f) => {
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
      [...files].forEach((f) => {
        this.encryptFiles.push(f);
      });
    },
    removeFileToEncrypt(index) {
      this.encryptFiles.splice(index, 1);
    },
    // Decryption methods
    startDecrypt(file) {
      this.decryptFile = file;
      this.wasm.Decryptor.new(file).then((decryptor) => {
        this.decryptor = decryptor;
      });
    },
    decryptWithPassphrase(passphrase) {
      let decryptor = this.decryptor;
      this.decryptor = null;

      // TODO:
      // - Handle if decryptor === null
      // - Disable Decrypt button while decrypting, re-enable on error

      decryptor.decrypt_with_passphrase(passphrase).then(
        (stream) => {
          this.decryptedStream = stream;
        },
        (e) => {
          this.reset();
          this.errorMsg = e;
        }
      );
    },
    downloadDecryptedFile() {
      // Default filename is the age-encrypted filename without the .age suffix.
      const fileName = this.decryptFile.name.slice(0, -4);

      this.downloadStream = window.streamSaver.createWriteStream(fileName);

      // Use the more optimized ReadableStream.pipeTo if available.
      if (window.WritableStream && this.decryptedStream.pipeTo) {
        return this.decryptedStream
          .pipeTo(this.downloadStream)
          .then(this.reset);
      }

      const reader = this.decryptedStream.getReader();
      const writer = this.downloadStream.getWriter();

      const pump = () =>
        reader
          .read()
          .then((res) =>
            res.done
              ? writer.close().then(this.reset)
              : writer.write(res.value).then(pump)
          );

      pump();
    },
    // File downloads happen in the browser, so navigating away from the page
    // will break any in-progress downloads.
    checkDownloads(evt) {
      if (this.fileDownloading) {
        evt.returnValue =
          "A file is still downloading; leaving will break the download. Are you sure you want to leave?";
      }
    },
    cancelDownloads() {
      if (this.fileDownloading) {
        this.downloadStream.abort();
        this.downloadStream = null;
      }
    },
  },
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
#footer {
  grid-area: footer;
}
.one-column-drop-zone {
  grid-template-columns: 10fr;
  grid-template-areas: "header" "drop-zone" "footer";
}
.one-column-drop-zone .drop-zone {
  height: 100%;
}
.two-column {
  grid-gap: 20px;
  grid-template-columns: 5fr 5fr;
  grid-template-rows: 1fr 3fr 2fr;
  grid-template-areas: "header header" "file-list details-pane" "drop-zone details-pane" "footer footer";
}
.one-column-decrypting {
  grid-template-columns: 10fr;
  grid-template-areas: "header" "details-pane" "footer";
}
</style>
