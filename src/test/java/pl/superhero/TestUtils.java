package pl.superhero;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import io.vertx.core.AsyncResult;
import io.vertx.core.Handler;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.net.URL;
import java.util.concurrent.CompletableFuture;
import static java.util.concurrent.TimeUnit.SECONDS;
import java.util.function.Consumer;
import org.apache.http.HttpEntity;
import static org.apache.http.entity.ContentType.APPLICATION_JSON;
import org.apache.http.entity.StringEntity;

/**
 *
 * @author witoldsz
 */
public class TestUtils {

  private static final ObjectMapper objectMapper = new ObjectMapper();

  public static JsonNode readNode(String s) {
    try {
      return objectMapper.readTree(s);
    } catch (IOException ex) {
      throw new UncheckedIOException(ex);
    }
  }

  public static JsonNode readNode(URL resource) {
    try {
      return objectMapper.readTree(resource);
    } catch (IOException ex) {
      throw new UncheckedIOException(ex);
    }
  }

  public static ObjectNode readObject(URL resource) {
    return (ObjectNode) readNode(resource);
  }

  public static ObjectNode readObject(String s) {
    return (ObjectNode) readNode(s);
  }

  public static HttpEntity jsonEntity(JsonNode json) {
    return new StringEntity(json.toString(), APPLICATION_JSON);
  }

  public static <T> T waitForCompletion(Consumer<Handler<AsyncResult<T>>> c) throws Exception {
    CompletableFuture<T> f = new CompletableFuture<>();
    c.accept(r -> {
      if (r.failed()) {
        f.completeExceptionally(r.cause());
      } else {
        f.complete(r.result());
      }
    });
    return f.get(5, SECONDS);
  }
}
