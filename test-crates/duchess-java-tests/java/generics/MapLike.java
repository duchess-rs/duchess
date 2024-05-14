package generics;

import java.util.HashMap;

public class MapLike<TT extends MapKey> {
    public MapLike() {
        this.storage = new HashMap<>();
    }

    private HashMap<TT, Object> storage;

    public void add(TT key, Object value) {
        storage.put(key, value);
    }

    public String toString() {
        StringBuilder sb = new StringBuilder();
        for (TT key : storage.keySet()) {
            sb.append(key + "=" + storage.get(key) + "\n");
        }
        return sb.toString();
    }
}
