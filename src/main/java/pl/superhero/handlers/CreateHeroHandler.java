package pl.superhero.handlers;

import io.vertx.core.Handler;
import io.vertx.core.json.JsonObject;
import io.vertx.ext.mongo.MongoClient;
import io.vertx.ext.web.RoutingContext;
import java.util.UUID;
import static java.util.Optional.ofNullable;

public class CreateHeroHandler implements Handler<RoutingContext> {

  private final MongoClient mongo;

  public CreateHeroHandler(MongoClient mongo) {
    this.mongo = mongo;
  }

  public void handle(RoutingContext routingContext) {
    JsonObject body = routingContext.getBodyAsJson();
    String id = ofNullable(body.getString("id")).orElseGet(() -> UUID.randomUUID().toString());
    mongo.save("heroes", new JsonObject()
      .put("_id", id)
      .put("name", body.getString("name"))
      .put("pseudonym", body.getString("pseudonym"))
      .put("publisher", body.getString("publisher"))
      .put("skills", body.getJsonArray("skills"))
      .put("allies", body.getJsonArray("allies"))
      .put("dateOfFirstAppearance", body.getString("dateOfFirstAppearance")),
      r -> {
        if (r.succeeded()) {
          routingContext.response().setStatusCode(201).end();
        } else {
          routingContext.fail(r.cause());
        }
    });
  }
}
