<template>
  <ul id="file-list">
    <li v-for="(file, index) in files" :key="file.name">
      {{ file.name }} ({{ file.size }} b)
      <button @click="removeFile(index)" title="Remove">X</button>
    </li>
  </ul>
</template>

<script>
export default {
  name: "FileList",
  props: ["files"],
  methods: {
    removeFile(index) {
      this.$emit("file-removed", index);
    }
  }
};
</script>

<style scoped>
#file-list {
  width: 100%;
  font-family: sans-serif;
  padding: 20px;
}
</style>
