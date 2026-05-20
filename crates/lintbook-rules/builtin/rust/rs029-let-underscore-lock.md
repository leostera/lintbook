---
id: RS029
lang: rust
---

Immediately dropping a lock guard with let _ provides no synchronization and should use a named binding.
