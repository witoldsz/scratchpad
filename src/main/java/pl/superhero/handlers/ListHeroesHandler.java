package pl.superhero.handlers;

import io.vertx.core.Handler;
import io.vertx.core.json.JsonArray;
import io.vertx.core.json.JsonObject;
import io.vertx.ext.mongo.MongoClient;
import io.vertx.ext.web.RoutingContext;

public class ListHeroesHandler implements Handler<RoutingContext> {

  private final MongoClient mongo;

  public ListHeroesHandler(MongoClient mongo) {
    this.mongo = mongo;
  }

  @Override
  public void handle(RoutingContext routingContext) {
    mongo.find("heroes", new JsonObject(), r -> {
      if (r.succeeded()) {
        if (r.result() == null) {
          routingContext.response().setStatusCode(404).end();
        } else {
          routingContext.response().end(new JsonArray(r.result()).toBuffer());
        }
      } else {
        routingContext.fail(r.cause());
      }
    });
  }
}
