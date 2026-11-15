package org.kmipkit.validator;

import org.kmipkit.validator.models.Models.CaseResult;
import org.kmipkit.validator.models.Models.CaseStatus;
import org.kmipkit.validator.models.Models.ServerCapabilities;
import org.kmipkit.validator.models.Models.TestCase;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class JsonLoader {
    public static ServerCapabilities loadCapabilities(Path path) throws IOException {
        String content = Files.readString(path);

        String serverId = extractString(content, "\"id\":\\s*\"([^\"]+)\"");
        if (serverId == null) serverId = "unknown-server";

        Set<String> ops = extractSupportedSet(content, "\"operations\":\\s*\\{", "\"managed_objects\":");
        Set<String> objs = extractSupportedSet(content, "\"managed_objects\":\\s*\\{", "\"attributes\":");
        Set<String> attrs = extractSupportedSet(content, "\"attributes\":\\s*\\{", "\"cryptographic_algorithms\":");

        return new ServerCapabilities(serverId, ops, objs, attrs);
    }

    public static List<TestCase> loadCatalog(Path path) throws IOException {
        String content = Files.readString(path);
        List<TestCase> cases = new ArrayList<>();

        int arrayStart = content.indexOf("\"test_cases\":");
        if (arrayStart == -1) return cases;
        int listStart = content.indexOf('[', arrayStart);
        if (listStart == -1) return cases;

        int depth = 0;
        int blockStart = -1;
        boolean inQuote = false;

        for (int i = listStart + 1; i < content.length(); i++) {
            char c = content.charAt(i);
            if (c == '"' && content.charAt(i - 1) != '\\') {
                inQuote = !inQuote;
                continue;
            }
            if (inQuote) continue;

            if (c == '{') {
                if (depth == 0) {
                    blockStart = i;
                }
                depth++;
            } else if (c == '}') {
                depth--;
                if (depth == 0 && blockStart != -1) {
                    String block = content.substring(blockStart, i + 1);
                    String caseId = extractString(block, "\"case_id\":\\s*\"([^\"]+)\"");
                    if (caseId != null) {
                        String op = extractString(block, "\"operation\":\\s*\"([^\"]+)\"");
                        String cat = extractString(block, "\"category\":\\s*\"([^\"]+)\"");
                        String obj = extractString(block, "\"object_type\":\\s*\"([^\"]+)\"");
                        String attr = extractString(block, "\"attribute\":\\s*\"([^\"]+)\"");

                        cases.add(new TestCase(caseId, cat != null ? cat : "", op != null ? op : "", obj != null ? obj : "", attr != null ? attr : ""));
                    }
                    blockStart = -1;
                }
            } else if (c == ']' && depth == 0) {
                break;
            }
        }

        return cases;
    }

    private static String extractString(String content, String regex) {
        Pattern pattern = Pattern.compile(regex);
        Matcher matcher = pattern.matcher(content);
        if (matcher.find()) {
            return matcher.group(1);
        }
        return null;
    }

    private static Set<String> extractSupportedSet(String content, String startRegex, String endMarker) {
        Set<String> result = new HashSet<>();
        Pattern startPattern = Pattern.compile(startRegex);
        Matcher startMatcher = startPattern.matcher(content);
        if (!startMatcher.find()) return result;

        int start = startMatcher.end();
        int end = content.indexOf(endMarker, start);
        if (end == -1) end = content.length();
        String section = content.substring(start, end);

        Pattern entryPattern = Pattern.compile("\"([^\"]+)\":\\s*\\{[^}]*?\"supported\":\\s*true", Pattern.DOTALL);
        Matcher entryMatcher = entryPattern.matcher(section);
        while (entryMatcher.find()) {
            result.add(entryMatcher.group(1));
        }
        return result;
    }
}
