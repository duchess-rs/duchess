package take_null;

public record TakeNullRecord(
    String field
) {
    public boolean isNull() {
        return this.field == null;
    }
}


