<template>
  <section>
    <b-field>
      <b-upload v-model="dropFiles" multiple drag-drop @input="filesChanged">
        <section class="section">
          <div class="content has-text-centered">
            <p>
              <b-icon icon="upload" size="is-large"> </b-icon>
            </p>
            <p>
              Drop files to encrypt
              <span v-if="!encrypting">or decrypt</span> here, or click to
              select.
            </p>
          </div>
        </section>
      </b-upload>
    </b-field>

    <div class="tags">
      <span
        v-for="(file, index) in dropFiles"
        :key="index"
        class="tag is-primary"
      >
        {{ file.name }}
        <button
          class="delete is-small"
          type="button"
          @click="deleteDropFile(index)"
        ></button>
      </span>
    </div>
  </section>
</template>

<script>
export default {
  name: "DropZone",
  props: ["dropFiles", "encrypting"],
  methods: {
    filesChanged() {
      this.$emit("files-changed");
    },
    deleteDropFile(index) {
      this.dropFiles.splice(index, 1);
      this.$emit("file-removed");
    },
  },
};
</script>
