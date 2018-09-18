package pl.superhero.handlers;

import io.vertx.core.Handler;
import io.vertx.ext.web.api.RequestParameters;
import io.vertx.ext.web.RoutingContext;

public class CreateHeroHandler implements Handler<RoutingContext> {

    public CreateHeroHandler(){

    }

    @Override
    public void handle(RoutingContext routingContext) {
        // Handle createHero
        routingContext.response().setStatusCode(501).setStatusMessage("Not Implemented").end();
    }

}