package org.kmipkit.validator;

import java.util.UUID;

public class AttemptContext {
    private final String runId;
    private final String language;
    private final String encoding;
    private final String apiLevel;
    private final String uniqueSuffix;

    public AttemptContext(String runId, String language, String encoding, String apiLevel) {
        this.runId = runId;
        this.language = language;
        this.encoding = encoding;
        this.apiLevel = apiLevel;
        this.uniqueSuffix = UUID.randomUUID().toString().substring(0, 8);
    }

    public String generateUniqueName(String prefix) {
        return String.format("%s-%s-%s-%s-%s", prefix, language, encoding, apiLevel, uniqueSuffix);
    }

    public String getRunId() { return runId; }
    public String getLanguage() { return language; }
    public String getEncoding() { return encoding; }
    public String getApiLevel() { return apiLevel; }
}
