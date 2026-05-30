# Debezium Connect + Groovy so the scripting-based Filter SMT works.
# (ENABLE_DEBEZIUM_SCRIPTING activates the Filter SMT, but the JSR-223 Groovy
# engine must be added to the worker classpath ourselves.)
FROM curlimages/curl:8.10.1 AS jars
WORKDIR /jars
ARG GROOVY=4.0.22
RUN base=https://repo1.maven.org/maven2/org/apache/groovy \
 && curl -fsSLO $base/groovy/$GROOVY/groovy-$GROOVY.jar \
 && curl -fsSLO $base/groovy-jsr223/$GROOVY/groovy-jsr223-$GROOVY.jar \
 && curl -fsSLO $base/groovy-json/$GROOVY/groovy-json-$GROOVY.jar

FROM debezium/connect:3.0.0.Final
ENV ENABLE_DEBEZIUM_SCRIPTING=true
COPY --from=jars /jars/groovy-*.jar /kafka/libs/
