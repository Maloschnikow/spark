<template>

  <div class="wrapper">
    <nav>
      <div class="left">

          <DefaultLink class="navbarLink" to="/">Home</DefaultLink>

          <DefaultLink class="navbarLink" to="/login">Login</DefaultLink>

          <DefaultLink class="navbarLink" to="/map">Map</DefaultLink>

          <DefaultLink class="navbarLink" to="https://github.com/Maloschnikow/spark" :external="true">Source Code</DefaultLink>

      </div>
    </nav>
  </div>

</template>

<style scoped>

nav {
  display: flex;
  font-weight: bold;
}

nav a.router-link-exact-active {
  color: var(--color-blue1);
}

nav a.router-link-exact-active:hover {
  background-color: transparent;
}

.navbarLink {
  margin: 1rem 1rem;
  display: inline-block;
}
</style>
