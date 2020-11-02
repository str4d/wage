<template>
  <div
    id="drop-zone"
    @dragenter="highlight"
    @dragover="highlight"
    @dragleave="unhighlight"
    @drop="handleFileDrop"
  >
    <p>Drag and drop files to encrypt or decrypt.</p>
    <input
      type="file"
      id="file-input"
      multiple="True"
      @change="handleFileInput"
    />
    <label class="button" for="file-input">Select some files</label>
  </div>
</template>

<script>
export default {
  name: "DropZone",
  methods: {
    highlight() {
      this.$el.classList.add("highlight");
    },
    unhighlight() {
      this.$el.classList.remove("highlight");
    },
    handleFileDrop(e) {
      this.unhighlight();
      this.handleFiles(e.dataTransfer.files);
    },
    handleFileInput(e) {
      this.handleFiles(e.target.files);
    },
    handleFiles(files) {
      if (!files) return;
      this.$emit("files-added", files);
    },
  },
};
</script>

<style scoped>
#drop-zone {
  border: 2px dashed #ccc;
  border-radius: 20px;
  font-family: sans-serif;
  padding: 20px;
}
#drop-zone.highlight {
  border-color: purple;
}
p {
  margin-top: 0;
}
#file-input {
  display: none;
}
</style>
