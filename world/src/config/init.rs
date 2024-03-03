//! Функции инициализации.

/*
/*
 *  init()
 *
 *  This is the overall initialization routine for the simulation.  It
 *  initialize the plant and agents.  If the population is not being
 *  seeded, the agents are all created randomly.  Otherwise, the agents
 *  are not random but instead read from the file.
 *
 */

void init( void )
{

  /* Initialize the landscape */
  bzero( (void *)landscape, sizeof(landscape) );

  bzero( (void *)bestAgent, sizeof(bestAgent) );

  /* Initialize the plant plane */
  for (plantCount = 0 ; plantCount < MAX_PLANTS ; plantCount++) {
    growPlant( plantCount );
  }

  if (seedPopulation == 0) {

    /* Randomly initialize the Agents */
    for (agentCount = 0 ; agentCount < MAX_AGENTS ; agentCount++) {

      if (agentCount < (MAX_AGENTS / 2)) {
        agents[agentCount].type = TYPE_HERBIVORE;
      } else {
        agents[agentCount].type = TYPE_CARNIVORE;
      }

      initAgent( &agents[agentCount] );

    }

  } else {

    /* In this case, we're seeding the population with the agents stored
     * within the agents.dat file.
     */

    FILE *fp;
    int offset;

    /* Try to seed the population from a file */
    fp = fopen(AGENTS, "r");

    fread( &bestAgent[0], sizeof( agentType ), 1, fp);
    fread( &bestAgent[1], sizeof( agentType ), 1, fp);

    for (agentCount = 0 ; agentCount < MAX_AGENTS ; agentCount++) {

      if (agentCount < MAX_AGENTS / 2) offset = 0;
      else offset = 1;

      memcpy( (void *)&agents[agentCount], (void *)&bestAgent[offset],
                sizeof(agentType) );
      findEmptySpot( &agents[agentCount] );

      agents[agentCount].energy = MAX_ENERGY;

      agentTypeCounts[agents[agentCount].type]++;

    }

  }

  return;
}

*/


/*
/*
 *  initAgent()
 *
 *  Initialize the agent passed by reference.
 *
 */

void initAgent( agentType *agent )
{
  int i;

  agent->energy = (MAX_ENERGY / 2);
  agent->age = 0;
  agent->generation = 1;

  agentTypeCounts[agent->type]++;

  findEmptySpot( agent );

  if (seedPopulation == 0) {
    for (i = 0 ; i < (MAX_INPUTS * MAX_OUTPUTS) ; i++) {
      agent->weight_oi[i] = getWeight();
    }

    for (i = 0 ; i < MAX_OUTPUTS ; i++) {
      agent->biaso[i] = getWeight();
    }
  }

  return;
}
*/