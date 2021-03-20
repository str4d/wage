<template>
  <section class="box">
    <b-tabs v-model="activeTab" type="is-toggle" position="is-centered">
      <b-tab-item label="Recipients">
        <div class="columns">
          <div class="column">
            <b-field :type="recipientFieldType" :message="recipientFieldError">
              <b-input
                v-model="recipient"
                placeholder="Recipient string"
                @blur="validateRecipientString"
            /></b-field>
          </div>
          <div class="column is-narrow">
            <b-button v-bind:disabled="addDisabled" @click="addRecipient"
              >Add</b-button
            >
          </div>
        </div>
        <div class="columns">
          <div class="column">
            <b-button
              type="is-danger"
              icon-left="trash"
              expanded
              v-bind:disabled="removeDisabled"
              @click="removeSelected"
              >Remove</b-button
            >
          </div>
          <div class="column">
            <b-button icon-left="key" expanded @click="newIdentity"
              >New identity</b-button
            >
          </div>
          <div class="column">
            <b-field class="file">
              <b-upload
                v-model="recipientsFiles"
                multiple
                expanded
                @input="validateRecipientsFile"
              >
                <a class="button is-fullwidth">
                  <b-icon icon="upload"></b-icon>
                  <span>Select file</span>
                </a>
              </b-upload>
            </b-field>
          </div>
        </div>
        <b-table
          :data="recipients"
          :columns="columns"
          :checked-rows.sync="checkedRows"
          checkable
        />
      </b-tab-item>
      <b-tab-item label="Passphrase">
        <b-field label="Passphrase">
          <b-input v-model="passphrase" type="password" password-reveal>
          </b-input>
        </b-field>
      </b-tab-item>
    </b-tabs>
    <b-button v-bind:disabled="encryptDisabled" @click="encryptFile"
      >Encrypt</b-button
    >
  </section>
</template>

<script>
import { saveAs } from "file-saver";

export default {
  name: "EncryptPane",
  props: ["wasm"],
  data() {
    return {
      activeTab: 0,
      recipient: "",
      recipientFieldError: "",
      recipientStrings: [],
      recipientsFiles: [],
      recipientsFilesParsed: [],
      columns: [
        {
          field: "label",
          label: "Recipients",
        },
      ],
      checkedRows: [],
      passphrase: "",
    };
  },
  computed: {
    recipientFieldType() {
      return this.recipientFieldError.length ? "is-danger" : "";
    },
    recipients() {
      return this.recipientsFiles
        .map((f, index) => {
          return { type: "file", index: index, label: f.name };
        })
        .concat(
          this.recipientStrings.map((r, index) => {
            return { type: "string", index: index, label: r };
          })
        );
    },
    // Button disabling
    addDisabled() {
      return this.recipientFieldError.length || !this.recipient.length;
    },
    removeDisabled() {
      return !this.checkedRows.length;
    },
    encryptDisabled() {
      return !(
        (this.activeTab === 0 && this.recipients.length) ||
        (this.activeTab === 1 && this.passphrase.length)
      );
    },
  },
  methods: {
    saveAs,
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
    validateRecipientsFile() {
      if (this.recipientsFiles.length) {
        this.wasm.Recipients.from_file(
          this.recipientsFiles[this.recipientsFiles.length - 1]
        )
          .then((r) => {
            this.recipientsFilesParsed.push(r);
          })
          .catch((e) => {
            this.recipientsFiles.pop();
            this.showError(e);
          });
      }
    },
    validateRecipientString() {
      if (this.recipient.length) {
        try {
          this.wasm.Recipients.from_recipient(this.recipient);
        } catch (err) {
          console.error(err);
          this.recipientFieldError = err;
          return;
        }
      }
      this.recipientFieldError = "";
    },
    // Recipients table updates
    addRecipient() {
      this.recipientStrings.push(this.recipient);
      this.recipient = "";
    },
    newIdentity() {
      let identity = this.wasm.X25519Identity.generate();

      let identityBlob = identity.write();
      this.saveAs(identityBlob, "identity.txt");

      let recipient = identity.recipient();
      this.recipientStrings.push(recipient);
    },
    removeSelected() {
      let files = this.checkedRows
        .filter((row) => row.type === "file")
        .map((row) => row.index);
      let strings = this.checkedRows
        .filter((row) => row.type === "string")
        .map((row) => row.index);

      this.recipientsFiles = this.recipientsFiles.filter((_, index) => {
        return !files.includes(index);
      });
      this.recipientsFilesParsed = this.recipientsFilesParsed.filter(
        (_, index) => {
          return !files.includes(index);
        }
      );
      this.recipientStrings = this.recipientStrings.filter((_, index) => {
        return !strings.includes(index);
      });

      this.checkedRows = [];
    },
    // Encrypt the file!
    encryptFile() {
      if (this.activeTab === 0) {
        const stringRecipients = this.recipientStrings.reduce((acc, r) => {
          if (acc === null) {
            return this.wasm.Recipients.from_recipient(r);
          } else {
            return acc.add_recipient(r);
          }
        }, null);

        const allRecipients = this.recipientsFilesParsed.reduce((acc, r) => {
          if (acc === null) {
            return r;
          } else {
            return acc.merge(r);
          }
        }, stringRecipients);

        this.$emit("encrypt-to-recipients", allRecipients);
      } else if (this.activeTab === 1) {
        this.$emit("encrypt-with-passphrase", this.passphrase);
      }
    },
  },
};
</script>
