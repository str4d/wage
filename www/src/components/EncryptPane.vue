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
              >Remove selected</b-button
            >
          </div>
          <div class="column"></div>
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
export default {
  name: "EncryptPane",
  props: ["wasm"],
  data() {
    return {
      activeTab: 0,
      recipient: "",
      recipientFieldError: "",
      recipientStrings: [],
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
      return this.recipientStrings.map((r, index) => {
        return { type: "string", index: index, label: r };
      });
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
    // Input validation
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
    removeSelected() {
      let strings = this.checkedRows
        .filter((row) => row.type === "string")
        .map((row) => row.index);

      this.recipientStrings = this.recipientStrings.filter((_, index) => {
        return !strings.includes(index);
      });

      this.checkedRows = [];
    },
    // Encrypt the file!
    encryptFile() {
      if (this.activeTab === 0) {
        const recipients = this.recipientStrings.reduce((acc, r) => {
          if (acc === null) {
            return this.wasm.Recipients.from_recipient(r);
          } else {
            return acc.add_recipient(r);
          }
        }, null);

        this.$emit("encrypt-to-recipients", recipients);
      } else if (this.activeTab === 1) {
        this.$emit("encrypt-with-passphrase", this.passphrase);
      }
    },
  },
};
</script>
