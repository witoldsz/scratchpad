package pl.superhero;

import io.vertx.core.AsyncResult;
import io.vertx.core.Future;
import io.vertx.core.Handler;
import java.util.function.Consumer;
import static io.vertx.core.Future.future;

public class Utils {

  public static <T> Future<T> $(Consumer<Handler<AsyncResult<T>>> completerConsumer) {
    Future<T> future = future();
    completerConsumer.accept(future.completer());
    return future;
  }
}
