<template>
  <div id="app" @dragover.prevent @drop.prevent>
    <h1 class="title">rage encrypt all the things!</h1>
    <div class="columns is-centered">
      <DropZone
        class="column"
        v-if="!decrypting"
        v-bind:dropFiles="dropFiles"
        v-on:files-changed="handleFiles"
        v-on:file-removed="removeFileToEncrypt"
      />
      <div class="column is-half" v-if="encrypting || decrypting">
        <FileInfo v-bind:fileIcon="fileIcon" v-on:reset-app="reset" />
        <EncryptPane
          id="details-pane"
          v-if="encrypting"
          v-on:encrypt-with-passphrase="encryptWithPassphrase"
        />
        <DecryptPane
          id="details-pane"
          v-if="decrypting"
          v-bind:fileDecrypted="fileDecrypted"
          v-bind:needPassphrase="needPassphrase"
          v-on:decrypt-with-passphrase="decryptWithPassphrase"
          v-on:download-file="downloadDecryptedFile"
        />
      </div>
    </div>
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
import FileInfo from "./components/FileInfo.vue";
import {
  getClassNameForFilename,
  getClassNameForMimeType,
} from "font-awesome-filetypes";

export default {
  name: "App",
  components: {
    DecryptPane,
    DropZone,
    EncryptPane,
    FileInfo,
  },
  data() {
    return {
      wasm: null,
      dropFiles: [],
      encryptMode: false,
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
      return this.encryptMode && !this.decrypting;
    },
    // Are we in "decrypting" mode?
    decrypting() {
      return this.decryptFile !== null;
    },
    // Icon matching the file we are encrypting or decrypting.
    fileIcon() {
      if (this.encrypting) {
        return (this.dropFiles.length > 1
          ? this.getClassNameForFilename("archive.zip")
          : this.getClassNameForMimeType(this.dropFiles[0].type)
        ).substring(3);
      } else if (this.decrypting) {
        // Default filename is the age-encrypted filename without the .age suffix.
        return this.getClassNameForFilename(
          this.decryptFile.name.slice(0, -4)
        ).substring(3);
      } else {
        return "file";
      }
    },
    // Do we need a passphrase from the user?
    needPassphrase() {
      return (
        this.decryptor !== null &&
        this.decryptor.requires() == this.wasm.DecryptorType.Passphrase
      );
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
    getClassNameForFilename,
    getClassNameForMimeType,
    // Reset application to initial state.
    reset() {
      this.dropFiles = [];
      this.encryptMode = false;
      this.decryptFile = null;
      this.decryptor = null;
      this.decryptedStream = null;
      this.downloadStream = null;
    },
    // Status messages.
    showError(e) {
      console.error(e);
      this.$buefy.toast.open({
        duration: 5000,
        message: e,
        position: "is-bottom",
        type: "is-danger",
      });
    },
    // This function is called by the drop zone, so only if we are starting out,
    // or are already encrypting.
    handleFiles() {
      if (!this.encrypting) {
        // Search for a decryptable file.
        var decryptIndex = [...this.dropFiles].findIndex((f) => {
          return f.name.endsWith(".age");
        });

        // Decide whether we are encrypting or decrypting.
        if (decryptIndex == -1) {
          this.encryptMode = true;
        } else {
          this.startDecrypt(this.dropFiles[decryptIndex]);
        }
      }
    },
    // Encryption methods
    removeFileToEncrypt() {
      if (!this.dropFiles.length) {
        this.reset();
      }
    },
    prepareEncryptStream(callback) {
      if (this.dropFiles.length > 1) {
        // TODO: Archive and encrypt.
        this.reset();
        this.showError("Encrypting multiple files is not yet supported");
      } else {
        // Default filename for a single file is the filename with an .age suffix.
        const fileName = this.dropFiles[0].name + ".age";
        callback(window.streamSaver.createWriteStream(fileName));
      }
    },
    encryptWithPassphrase(passphrase) {
      this.prepareEncryptStream((outputStream) => {
        this.wasm.Encryptor.with_user_passphrase(passphrase)
          .wrap_output(outputStream)
          .then((sink) => {
            this.downloadStream = sink;
            this.encryptSingleFile();
          });
      });
    },
    encryptSingleFile() {
      let fileStream = this.dropFiles[0].stream();

      // Use the more optimized ReadableStream.pipeTo if available.
      if (window.WritableStream && fileStream.pipeTo) {
        return fileStream.pipeTo(this.downloadStream).then(this.reset);
      }

      const reader = fileStream.getReader();
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
          this.showError(e);
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
</style>
