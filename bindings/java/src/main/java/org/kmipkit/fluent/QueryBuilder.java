package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP Query request. */
public final class QueryBuilder extends TypedOperationBuilder<QueryBuilder> {
    private final List<Integer> functions = new ArrayList<>();
    private List<String> objectGroups;

    QueryBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected QueryBuilder self() { return this; }

    public QueryBuilder function(int queryFunction) {
        ensureOpen();
        functions.add(queryFunction);
        return this;
    }

    public QueryBuilder functions(int... queryFunctions) {
        ensureOpen();
        if (queryFunctions == null) throw new InvalidInputException("query functions are required");
        for (int fn : queryFunctions) functions.add(fn);
        return this;
    }

    public QueryBuilder objectGroups(List<String> groups) {
        ensureOpen();
        objectGroups = groups == null ? null : new ArrayList<>(groups);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        int[] fns = functions.isEmpty() ? new int[] {1} : functions.stream().mapToInt(Integer::intValue).toArray();
        String[] groups = objectGroups == null ? null : objectGroups.toArray(String[]::new);
        return validated(ClientBatchItem.query(fns, groups));
    }
}
