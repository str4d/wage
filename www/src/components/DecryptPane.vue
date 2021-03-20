<template>
  <section class="box">
    <div v-if="needIdentities">
      <div class="columns">
        <div class="column">
          <b-button
            type="is-danger"
            icon-left="trash"
            expanded
            v-bind:disabled="removeDisabled"
            @click="removeSelected"
            >Remove selected</b-button
          >
        </div>
        <div class="column">
          <b-field class="file">
            <b-upload
              v-model="identitiesFiles"
              multiple
              expanded
              @input="validateIdentitiesFile"
            >
              <a class="button is-fullwidth">
                <b-icon icon="upload"></b-icon>
                <span>Select identities file</span>
              </a>
            </b-upload>
          </b-field>
        </div>
      </div>
      <b-table
        :data="identities"
        :columns="columns"
        :checked-rows.sync="checkedRows"
        checkable
      />
    </div>
    <b-field label="Passphrase" v-if="!fileDecrypted && needPassphrase">
      <b-input type="password" v-model="passphrase" password-reveal> </b-input>
    </b-field>
    <b-button
      v-if="!fileDecrypted"
      v-bind:disabled="decryptDisabled"
      @click="decryptFile"
      >Decrypt</b-button
    >
    <b-button v-if="fileDecrypted" @click="downloadFile">Download</b-button>
  </section>
</template>

<script>
export default {
  name: "DecryptPane",
  props: ["wasm", "fileDecrypted", "needIdentities", "needPassphrase"],
  data() {
    return {
      identitiesFiles: [],
      identities: [],
      columns: [
        {
          field: "label",
          label: "Identities",
        },
      ],
      checkedRows: [],
      passphrase: "",
    };
  },
  computed: {
    // Button disabling
    removeDisabled() {
      return !this.checkedRows.length;
    },
    decryptDisabled() {
      return !(
        (this.needIdentities && this.identities.length) ||
        (this.needPassphrase && this.passphrase.length)
      );
    },
  },
  methods: {
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
    // Input validation
    validateIdentitiesFile() {
      if (this.identitiesFiles.length) {
        let index = this.identitiesFiles.length - 1;
        let f = this.identitiesFiles[index];

        this.wasm.Identities.from_file(f)
          .then((i) => {
            this.identities.push({ index: index, label: f.name, file: i });
          })
          .catch((e) => {
            this.identitiesFiles.pop();
            this.showError(e);
          });
      }
    },
    // Identities table updates
    removeSelected() {
      let files = this.checkedRows.map((row) => row.index);

      this.identitiesFiles = this.identitiesFiles.filter((_, index) => {
        return !files.includes(index);
      });
      this.identities = this.identities.filter((_, index) => {
        return !files.includes(index);
      });

      this.checkedRows = [];
    },
    // Decrypt the file!
    decryptFile() {
      if (this.needIdentities) {
        const identities = this.identities.reduce((acc, i) => {
          if (acc === null) {
            return i.file;
          } else {
            return acc.merge(i.file);
          }
        }, null);

        this.$emit("decrypt-with-identities", identities);
      } else if (this.needPassphrase) {
        this.$emit("decrypt-with-passphrase", this.passphrase);
      }
    },
    downloadFile() {
      this.$emit("download-file");
    },
  },
};
</script>
