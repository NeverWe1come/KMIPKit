package org.kmipkit.ttlv;

/** KMIP TTLV item types; numeric codes follow the KMIP 2.1 TTLV encoding. */
public enum TtlvItemType {
    Structure(1),
    Integer(2),
    LongInteger(3),
    BigInteger(4),
    Enumeration(5),
    Boolean(6),
    TextString(7),
    ByteString(8),
    DateTime(9),
    Interval(10),
    DateTimeExtended(11);

    private final int code;

    TtlvItemType(int code) {
        this.code = code;
    }

    public int code() {
        return code;
    }

    public static TtlvItemType fromCode(int code) {
        for (TtlvItemType value : values()) {
            if (value.code == code) {
                return value;
            }
        }
        throw new org.kmipkit.InvalidInputException("unknown TTLV item type");
    }
}
