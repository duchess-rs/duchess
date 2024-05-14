package generics;

import java.util.HashMap;

public class MapLike<T extends MapKey> {
    public MapLike() {
        this.storage = new HashMap<>();
    }

    private HashMap<T, Object> storage;

    public void add(T key, Object value) {
        storage.put(key, value);
    }

    public String toString() {
        StringBuilder sb = new StringBuilder();
        for (T key : storage.keySet()) {
            sb.append(key + "=" + storage.get(key) + "\n");
        }
        return sb.toString();
    }
}
