package generics;

import java.util.HashMap;
import java.util.TreeSet;
import java.util.Comparator;

public class MapLike<TT extends MapKey> {
    public MapLike() {
        this.storage = new HashMap<>();
    }

    private HashMap<TT, Object> storage;

    public void add(TT key, Object value) {
        storage.put(key, value);
    }

    // Note: although Java supports shadowing generics, Duchess currently does not.
    public <T extends MapKey> void methodGeneric(T key, Object value) {
        storage.put((TT) key, value);
    }

    public String toString() {
        StringBuilder sb = new StringBuilder();
        TreeSet<TT> sortedKeys = new TreeSet<TT>();
sortedKeys.addAll(storage.keySet());

        for (TT key : sortedKeys) {
            sb.append(key + "=" + storage.get(key) + "\n");
        }
        return sb.toString();
    }
}
